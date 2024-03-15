use super::{
    payload::{Payload, ValidationError},
    store::Store,
    Invocation,
};
use crate::ability::arguments::Named;
use crate::ability::command::ToCommand;
use crate::ability::parse::ParseAbility;
use crate::invocation::payload::PayloadBuilder;
use crate::{
    ability::{self, arguments, parse::ParseAbilityError, ucan::revoke::Revoke},
    crypto::{
        signature::{self, Envelope},
        varsig, Nonce,
    },
    delegation,
    did::{self, Did},
    time::Timestamp,
};
use enum_as_inner::EnumAsInner;
use libipld_core::{
    cid::Cid,
    codec::{Codec, Encode},
    ipld::Ipld,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt,
    marker::PhantomData,
};
use thiserror::Error;
use web_time::SystemTime;

#[derive(Debug)]
pub struct Agent<
    'a,
    S: Store<T, DID, V, C>,
    D: delegation::store::Store<DID, V, C>,
    T: ToCommand = ability::preset::Preset,
    DID: Did = did::preset::Verifier,
    V: varsig::Header<C> + Clone = varsig::header::Preset,
    C: Codec + Into<u64> + TryFrom<u64> = varsig::encoding::Preset,
> {
    /// The agent's [`DID`].
    pub did: &'a DID,

    /// A [`delegation::Store`][delegation::store::Store].
    pub delegation_store: D,

    /// A [`Store`][Store] for the agent's [`Invocation`]s.
    pub invocation_store: S,

    signer: &'a <DID as Did>::Signer,
    marker: PhantomData<(T, V, C)>,
}

impl<'a, T, DID, S, D, V, C> Agent<'a, S, D, T, DID, V, C>
where
    Ipld: Encode<C>,
    T: ToCommand + Clone + ParseAbility,
    Named<Ipld>: From<T>,
    Payload<T, DID>: TryFrom<Named<Ipld>>,
    delegation::Payload<DID>: Clone,
    DID: Did + Clone,
    S: Store<T, DID, V, C>,
    D: delegation::store::Store<DID, V, C>,
    V: varsig::Header<C> + Clone,
    C: Codec + Into<u64> + TryFrom<u64>,
    <S as Store<T, DID, V, C>>::InvocationStoreError: fmt::Debug,
    <D as delegation::store::Store<DID, V, C>>::DelegationStoreError: fmt::Debug,
{
    pub fn new(
        did: &'a DID,
        signer: &'a <DID as Did>::Signer,
        invocation_store: S,
        delegation_store: D,
    ) -> Self {
        Self {
            did,
            invocation_store,
            delegation_store,
            signer,
            marker: PhantomData,
        }
    }

    pub fn invoke(
        &mut self,
        audience: Option<DID>,
        subject: DID,
        ability: T,
        metadata: BTreeMap<String, Ipld>,
        cause: Option<Cid>,
        expiration: Option<Timestamp>,
        issued_at: Option<Timestamp>,
        now: SystemTime,
        varsig_header: V,
    ) -> Result<Invocation<T, DID, V, C>, InvokeError<D::DelegationStoreError>> {
        let proofs = self
            .delegation_store
            .get_chain(&self.did, &Some(subject.clone()), "/".into(), vec![], now) // FIXME
            .map_err(InvokeError::DelegationStoreError)?
            .map(|chain| chain.map(|(cid, _)| cid).into())
            .unwrap_or(vec![]);

        let mut seed = vec![];

        let payload = Payload {
            issuer: self.did.clone(),
            subject,
            audience,
            ability,
            proofs,
            metadata,
            nonce: Nonce::generate_12(&mut seed),
            cause,
            expiration,
            issued_at,
        };

        Ok(Invocation::try_sign(&self.signer, varsig_header, payload)
            .map_err(InvokeError::SignError)?)
    }

    // pub fn invoke_promise(
    //     &mut self,
    //     audience: Option<&DID>,
    //     subject: DID,
    //     ability: T::Promised,
    //     metadata: BTreeMap<String, Ipld>,
    //     cause: Option<Cid>,
    //     expiration: Option<Timestamp>,
    //     issued_at: Option<Timestamp>,
    //     now: SystemTime,
    //     varsig_header: V,
    // ) -> Result<
    //     Invocation<T::Promised, DID, V, C>,
    //     InvokeError<
    //         D::DelegationStoreError,
    //         ParseAbilityError<()>, // FIXME errs
    //     >,
    // >
    // where
    //     Named<Ipld>: From<T::Promised>,
    //     Payload<T::Promised, DID>: TryFrom<Named<Ipld>>,
    // {
    //     let proofs = self
    //         .delegation_store
    //         .get_chain(self.did, &Some(subject.clone()), "/".into(), vec![], now)
    //         .map_err(InvokeError::DelegationStoreError)?
    //         .map(|chain| chain.map(|(cid, _)| cid).into())
    //         .unwrap_or(vec![]);

    //     let mut seed = vec![];

    //     let payload = Payload {
    //         issuer: self.did.clone(),
    //         subject,
    //         audience: audience.cloned(),
    //         ability,
    //         proofs,
    //         metadata,
    //         nonce: Nonce::generate_12(&mut seed),
    //         cause,
    //         expiration,
    //         issued_at,
    //     };

    //     Ok(Invocation::try_sign(self.signer, varsig_header, payload)
    //         .map_err(InvokeError::SignError)?)
    // }

    pub fn receive(
        &mut self,
        invocation: Invocation<T, DID, V, C>,
    ) -> Result<Recipient<Payload<T, DID>>, ReceiveError<T, DID, D::DelegationStoreError, S, V, C>>
    where
        arguments::Named<Ipld>: From<T>,
        Payload<T, DID>: TryFrom<Named<Ipld>>,
        Invocation<T, DID, V, C>: Clone + Encode<C>,
    {
        self.generic_receive(invocation, &SystemTime::now())
    }

    pub fn generic_receive(
        &mut self,
        invocation: Invocation<T, DID, V, C>,
        now: &SystemTime,
    ) -> Result<Recipient<Payload<T, DID>>, ReceiveError<T, DID, D::DelegationStoreError, S, V, C>>
    where
        arguments::Named<Ipld>: From<T>,
        Payload<T, DID>: TryFrom<Named<Ipld>>,
        Invocation<T, DID, V, C>: Clone + Encode<C>,
    {
        let cid: Cid = invocation.cid().map_err(ReceiveError::EncodingError)?;

        self.invocation_store
            .put(cid.clone(), invocation.clone())
            .map_err(ReceiveError::InvocationStoreError)?;

        let proof_payloads: Vec<&delegation::Payload<DID>> = self
            .delegation_store
            .get_many(&invocation.proofs())
            .map_err(ReceiveError::DelegationStoreError)?
            .iter()
            .map(|d| &d.payload)
            .collect();

        let _ = &invocation
            .payload
            .check(proof_payloads, now)
            .map_err(ReceiveError::ValidationError)?;

        Ok(if invocation.normalized_audience() != self.did {
            Recipient::Other(invocation.payload)
        } else {
            Recipient::You(invocation.payload)
        })
    }

    // pub fn revoke(
    //     &mut self,
    //     subject: DID,
    //     cause: Option<Cid>,
    //     cid: Cid,
    //     now: Timestamp,
    //     varsig_header: V,
    //     // FIXME return type
    // ) -> Result<Invocation<T, DID, V, C>, ()>
    // where
    //     Named<Ipld>: From<T>,
    //     T: From<Revoke>,
    //     Payload<T, DID>: TryFrom<Named<Ipld>>,
    // {
    //     let ability: T = Revoke { ucan: cid.clone() }.into();
    //     let proofs = if &subject == self.did {
    //         vec![]
    //     } else {
    //         todo!("update to latest trait interface"); // FIXME
    //                                                    // self.delegation_store
    //                                                    //     .get_chain(&subject, &Some(self.did.clone()), vec![], now.into())
    //                                                    //     .map_err(|_| ())?
    //                                                    //     .map(|chain| chain.map(|(index_cid, _)| index_cid).into())
    //                                                    //     .unwrap_or(vec![])
    //     };

    //     let payload = Payload {
    //         issuer: self.did.clone(),
    //         subject: self.did.clone(),
    //         audience: Some(self.did.clone()),
    //         ability,
    //         proofs,
    //         cause,
    //         metadata: BTreeMap::new(),
    //         nonce: Nonce::generate_12(&mut vec![]),
    //         expiration: None,
    //         issued_at: None,
    //     };

    //     let invocation =
    //         Invocation::try_sign(self.signer, varsig_header, payload).map_err(|_| ())?;

    //     self.delegation_store.revoke(cid).map_err(|_| ())?;
    //     Ok(invocation)
    // }
}

#[derive(Debug, PartialEq, Clone, EnumAsInner)]
pub enum Recipient<T> {
    // FIXME change to status?
    You(T),
    Other(T),
    Unresolved(Cid),
}

#[derive(Debug, Error, EnumAsInner)]
pub enum ReceiveError<
    T,
    DID: Did,
    D,
    S: Store<T, DID, V, C>,
    V: varsig::Header<C>,
    C: Codec + TryFrom<u64> + Into<u64>,
> where
    <S as Store<T, DID, V, C>>::InvocationStoreError: fmt::Debug,
{
    #[error("encoding error: {0}")]
    EncodingError(#[from] libipld_core::error::Error),

    #[error("signature verification error: {0}")]
    SigVerifyError(#[from] signature::ValidateError),

    #[error("invocation store error: {0}")]
    InvocationStoreError(#[source] <S as Store<T, DID, V, C>>::InvocationStoreError),

    #[error("delegation store error: {0}")]
    DelegationStoreError(#[source] D),

    #[error("delegation validation error: {0}")]
    ValidationError(#[source] ValidationError<DID>),
}

#[derive(Debug, Error)]
pub enum InvokeError<D> {
    #[error("delegation store error: {0}")]
    DelegationStoreError(#[source] D),

    #[error("store error: {0}")]
    SignError(#[source] signature::SignError),
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions as pretty;
    use rand::thread_rng;
    use std::ops::Add;
    use std::ops::Sub;
    use std::time::Duration;
    use testresult::TestResult;

    fn gen_did() -> (crate::did::preset::Verifier, crate::did::preset::Signer) {
        let sk = ed25519_dalek::SigningKey::generate(&mut thread_rng());
        let verifier =
            crate::did::preset::Verifier::Key(crate::did::key::Verifier::EdDsa(sk.verifying_key()));
        let signer = crate::did::preset::Signer::Key(crate::did::key::Signer::EdDsa(sk));

        (verifier, signer)
    }

    fn setup_agent<'a>(
        did: &'a crate::did::preset::Verifier,
        signer: &'a crate::did::preset::Signer,
    ) -> Agent<'a, crate::invocation::store::MemoryStore, crate::delegation::store::MemoryStore>
    {
        let inv_store = crate::invocation::store::MemoryStore::default();
        let del_store = crate::delegation::store::MemoryStore::default();

        crate::invocation::agent::Agent::new(did, signer, inv_store, del_store)
    }

    fn setup_valid_time() -> (Timestamp, Timestamp, Timestamp) {
        let now = SystemTime::UNIX_EPOCH.add(Duration::from_secs(60 * 60 * 24 * 30));
        let exp = now.add(std::time::Duration::from_secs(60));
        let nbf = now.sub(std::time::Duration::from_secs(60));

        (
            nbf.try_into().expect("valid nbf time"),
            now.try_into().expect("valid now time"),
            exp.try_into().expect("valid exp time"),
        )
    }

    mod receive {
        use super::*;
        use crate::ability::crud::read::Read;
        use crate::crypto::varsig;

        #[test_log::test]
        fn test_invoker_is_sub_implicit_aud() -> TestResult {
            let (_nbf, now, exp) = setup_valid_time();
            let (server, server_signer) = gen_did();
            let mut agent = setup_agent(&server, &server_signer);

            let invocation = agent.invoke(
                None,
                agent.did.clone(),
                Read {
                    path: None,
                    args: None,
                }
                .into(),
                BTreeMap::new(),
                None,
                Some(exp.try_into()?),
                Some(now.try_into()?),
                now.into(),
                varsig::header::Preset::EdDsa(varsig::header::EdDsaHeader {
                    codec: varsig::encoding::Preset::DagCbor,
                }),
            )?;

            let observed = agent.generic_receive(invocation.clone(), &now.into())?;
            pretty::assert_eq!(observed, Recipient::You(invocation.payload));
            Ok(())
        }

        #[test_log::test]
        fn test_invoker_is_sub_and_aud() -> TestResult {
            let (_nbf, now, exp) = setup_valid_time();
            let (server, server_signer) = gen_did();
            let mut agent = setup_agent(&server, &server_signer);

            let invocation = agent.invoke(
                Some(agent.did.clone()),
                agent.did.clone(),
                Read {
                    path: None,
                    args: None,
                }
                .into(),
                BTreeMap::new(),
                None,
                Some(exp.try_into()?),
                Some(now.try_into()?),
                now.into(),
                varsig::header::Preset::EdDsa(varsig::header::EdDsaHeader {
                    codec: varsig::encoding::Preset::DagCbor,
                }),
            )?;

            let observed = agent.generic_receive(invocation.clone(), &now.into())?;
            pretty::assert_eq!(observed, Recipient::You(invocation.payload));
            Ok(())
        }

        #[test_log::test]
        fn test_other_recipient() -> TestResult {
            let (_nbf, now, exp) = setup_valid_time();
            let (server, server_signer) = gen_did();
            let mut agent = setup_agent(&server, &server_signer);

            let (not_server, _) = gen_did();

            let invocation = agent.invoke(
                Some(not_server),
                agent.did.clone(),
                Read {
                    path: None,
                    args: None,
                }
                .into(),
                BTreeMap::new(),
                None,
                Some(exp.try_into()?),
                Some(now.try_into()?),
                now.into(),
                varsig::header::Preset::EdDsa(varsig::header::EdDsaHeader {
                    codec: varsig::encoding::Preset::DagCbor,
                }),
            )?;

            let observed = agent.generic_receive(invocation.clone(), &now.into())?;
            pretty::assert_eq!(observed, Recipient::Other(invocation.payload));
            Ok(())
        }

        #[test_log::test]
        fn test_expired() -> TestResult {
            let (past, now, _exp) = setup_valid_time();
            let (server, server_signer) = gen_did();
            let mut agent = setup_agent(&server, &server_signer);

            let (not_server, _) = gen_did();

            let invocation = agent.invoke(
                Some(not_server),
                agent.did.clone(),
                Read {
                    path: None,
                    args: None,
                }
                .into(),
                BTreeMap::new(),
                None,
                Some(past.try_into()?),
                Some(now.try_into()?),
                now.into(),
                varsig::header::Preset::EdDsa(varsig::header::EdDsaHeader {
                    codec: varsig::encoding::Preset::DagCbor,
                }),
            )?;

            let observed = agent.generic_receive(invocation.clone(), &now.into());
            pretty::assert_eq!(
                observed
                    .unwrap_err()
                    .as_validation_error()
                    .ok_or("not a validation error")?,
                &ValidationError::Expired
            );
            Ok(())
        }
    }
}
