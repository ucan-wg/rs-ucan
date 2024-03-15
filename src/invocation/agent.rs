use super::{
    payload::{Payload, ValidationError},
    store::Store,
    Invocation,
};
use crate::ability::arguments::Named;
use crate::ability::command::ToCommand;
use crate::ability::parse::ParseAbility;
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

        Ok(if *invocation.audience() != Some(self.did.clone()) {
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
    use rand::thread_rng;
    use testresult::TestResult;

    fn setup_agent<'a>(
        did: &'a crate::did::preset::Verifier,
        signer: &'a crate::did::preset::Signer,
    ) -> Agent<'a, crate::invocation::store::MemoryStore, crate::delegation::store::MemoryStore>
    {
        let inv_store = crate::invocation::store::MemoryStore::default();
        let del_store = crate::delegation::store::MemoryStore::default();

        crate::invocation::agent::Agent::new(did, signer, inv_store, del_store)
    }

    mod receive {
        use super::*;
        use crate::ability::crud::{read::Read, Crud};
        use crate::ability::preset::Preset;
        use crate::crypto::varsig;

        #[test_log::test]
        fn test_happy_path() -> TestResult {
            let server_sk = ed25519_dalek::SigningKey::generate(&mut thread_rng());
            let server_signer =
                crate::did::preset::Signer::Key(crate::did::key::Signer::EdDsa(server_sk.clone()));

            let server = crate::did::preset::Verifier::Key(crate::did::key::Verifier::EdDsa(
                server_sk.verifying_key(),
            ));

            let mut agent = setup_agent(&server, &server_signer);
            let invocation = agent.invoke(
                None,
                agent.did.clone(),
                // FIXME flatten
                Preset::Crud(Crud::Read(Read {
                    path: None,
                    args: None,
                })),
                BTreeMap::new(),
                None,
                None,
                None,
                SystemTime::now(),
                varsig::header::Preset::EdDsa(varsig::header::EdDsaHeader {
                    codec: varsig::encoding::Preset::DagCbor,
                }),
            )?;

            let unknown_sk = ed25519_dalek::SigningKey::generate(&mut thread_rng());
            let unknown_did = crate::did::preset::Verifier::Key(crate::did::key::Verifier::EdDsa(
                unknown_sk.verifying_key(),
            ));

            agent.receive(invocation)?;

            Ok(())
        }
    }
}
