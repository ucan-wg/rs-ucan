use super::{
    payload::{Payload, ValidationError},
    store::Store,
    Invocation,
};
use crate::ability::arguments::Named;
use crate::ability::command::ToCommand;
use crate::ability::parse::ParseAbility;
use crate::delegation::Delegation;
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
use std::{collections::BTreeMap, fmt, marker::PhantomData};
use thiserror::Error;
use web_time::SystemTime;

#[derive(Debug)]
pub struct Agent<
    S: Store<T, DID, V, C>,
    D: delegation::store::Store<DID, V, C>,
    T: ToCommand = ability::preset::Preset,
    DID: Did + Clone = did::preset::Verifier,
    V: varsig::Header<C> + Clone = varsig::header::Preset,
    C: Codec + Into<u64> + TryFrom<u64> = varsig::encoding::Preset,
> where
    Ipld: Encode<C>,
    delegation::Payload<DID>: TryFrom<Named<Ipld>>,
    Named<Ipld>: From<delegation::Payload<DID>>,
{
    /// The agent's [`DID`].
    pub did: DID,

    /// A [`delegation::Store`][delegation::store::Store].
    pub delegation_store: D,

    /// A [`Store`][Store] for the agent's [`Invocation`]s.
    pub invocation_store: S,

    signer: <DID as Did>::Signer,
    marker: PhantomData<(T, V, C)>,
}

impl<T, DID, S, D, V, C> Agent<S, D, T, DID, V, C>
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
    delegation::Payload<DID>: TryFrom<Named<Ipld>>,
    Named<Ipld>: From<delegation::Payload<DID>>,
{
    pub fn new(
        did: DID,
        signer: <DID as Did>::Signer,
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
        &self,
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
        let proofs = if subject == self.did {
            vec![]
        } else {
            self.delegation_store
                .get_chain(
                    &self.did,
                    &subject.clone(),
                    ability.to_command(),
                    vec![],
                    now,
                )
                .map_err(InvokeError::DelegationStoreError)?
                .map(|chain| chain.map(|(cid, _)| cid).into())
                .unwrap_or(vec![]) // FIXME
        };

        let payload = Payload {
            issuer: self.did.clone(),
            subject,
            audience,
            ability,
            proofs,
            metadata,
            nonce: Nonce::generate_12(&mut vec![]),
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
        &self,
        invocation: Invocation<T, DID, V, C>,
    ) -> Result<Recipient<Payload<T, DID>>, ReceiveError<T, DID, D::DelegationStoreError, S, V, C>>
    where
        arguments::Named<Ipld>: From<T>,
        Payload<T, DID>: TryFrom<Named<Ipld>>,
        Invocation<T, DID, V, C>: Clone + Encode<C>,
    {
        self.generic_receive(invocation, SystemTime::now())
    }

    pub fn generic_receive(
        &self,
        invocation: Invocation<T, DID, V, C>,
        now: SystemTime,
    ) -> Result<Recipient<Payload<T, DID>>, ReceiveError<T, DID, D::DelegationStoreError, S, V, C>>
    where
        arguments::Named<Ipld>: From<T>,
        Payload<T, DID>: TryFrom<Named<Ipld>>,
        Invocation<T, DID, V, C>: Clone + Encode<C>,
    {
        let cid: Cid = invocation.cid().map_err(ReceiveError::EncodingError)?;

        invocation
            .validate_signature()
            .map_err(ReceiveError::SigVerifyError)?;

        // FIXME validate signature directly in inv store

        self.invocation_store
            .put(cid.clone(), invocation.clone())
            .map_err(ReceiveError::InvocationStoreError)?;

        let proofs = &self
            .delegation_store
            .get_many(&invocation.proofs())
            .map_err(ReceiveError::DelegationStoreError)?;
        let proof_payloads: Vec<&delegation::Payload<DID>> = proofs
            .iter()
            .zip(invocation.proofs().iter())
            .map(|(d, cid)| {
                Ok(&d
                    .as_ref()
                    .ok_or(ReceiveError::MissingDelegation(*cid))?
                    .payload)
            })
            .collect::<Result<_, ReceiveError<T, DID, D::DelegationStoreError, S, V, C>>>()?;

        let _ = &invocation
            .payload
            .check(proof_payloads, now)
            .map_err(ReceiveError::ValidationError)?;

        Ok(if invocation.normalized_audience() != &self.did {
            Recipient::Other(invocation.payload)
        } else {
            Recipient::You(invocation.payload)
        })
    }

    // pub fn revoke(
    //     &self,
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
    #[error("missing delegation: {0}")]
    MissingDelegation(Cid),

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
    use crate::ability::crud::read::Read;
    use crate::crypto::varsig;
    use crate::crypto::varsig::encoding;
    use crate::crypto::varsig::header;
    use crate::invocation::{payload::ValidationError, Agent};
    use crate::{
        ability::{arguments::Named, command::Command},
        crypto::signature::Envelope,
        delegation::store::Store,
        invocation::promise::{CantResolve, Resolvable},
        ipld,
    };
    use libipld_core::{cid::Cid, ipld::Ipld};
    use pretty_assertions as pretty;
    use rand::thread_rng;
    use std::ops::{Add, Sub};
    use std::time::{Duration, SystemTime};
    use testresult::TestResult;

    #[derive(Debug, Clone, PartialEq)]
    pub struct AccountManage;

    impl Command for AccountManage {
        const COMMAND: &'static str = "/account/info";
    }

    impl From<AccountManage> for Named<Ipld> {
        fn from(_: AccountManage) -> Self {
            Default::default()
        }
    }

    impl TryFrom<Named<Ipld>> for AccountManage {
        type Error = ();

        fn try_from(args: Named<Ipld>) -> Result<Self, ()> {
            if args == Default::default() {
                Ok(AccountManage)
            } else {
                Err(())
            }
        }
    }

    impl From<AccountManage> for Named<ipld::Promised> {
        fn from(_: AccountManage) -> Self {
            Default::default()
        }
    }

    impl TryFrom<Named<ipld::Promised>> for AccountManage {
        type Error = ();

        fn try_from(args: Named<ipld::Promised>) -> Result<Self, ()> {
            if args == Default::default() {
                Ok(AccountManage)
            } else {
                Err(())
            }
        }
    }

    impl Resolvable for AccountManage {
        type Promised = AccountManage;

        fn try_resolve(promised: Self::Promised) -> Result<Self, CantResolve<Self>> {
            Ok(promised)
        }
    }

    fn gen_did() -> (crate::did::preset::Verifier, crate::did::preset::Signer) {
        let sk = ed25519_dalek::SigningKey::generate(&mut thread_rng());
        let verifier =
            crate::did::preset::Verifier::Key(crate::did::key::Verifier::EdDsa(sk.verifying_key()));
        let signer = crate::did::preset::Signer::Key(crate::did::key::Signer::EdDsa(sk));

        (verifier, signer)
    }

    fn setup_agent(
    ) -> Agent<crate::invocation::store::MemoryStore, crate::delegation::store::MemoryStore> {
        let (did, signer) = gen_did();
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
        use assert_matches::assert_matches;

        #[test_log::test]
        fn test_invoker_is_sub_implicit_aud() -> TestResult {
            let (_nbf, now, exp) = setup_valid_time();
            let agent = setup_agent();

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

            let observed = agent.generic_receive(invocation.clone(), now.into())?;
            pretty::assert_eq!(observed, Recipient::You(invocation.payload));
            Ok(())
        }

        #[test_log::test]
        fn test_invoker_is_sub_and_aud() -> TestResult {
            let (_nbf, now, exp) = setup_valid_time();
            let agent = setup_agent();

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
                header::Preset::EdDsa(header::EdDsaHeader {
                    codec: encoding::Preset::DagCbor,
                }),
            )?;

            let observed = agent.generic_receive(invocation.clone(), now.into())?;
            pretty::assert_eq!(observed, Recipient::You(invocation.payload));

            Ok(())
        }

        #[test_log::test]
        fn test_other_recipient() -> TestResult {
            let (_nbf, now, exp) = setup_valid_time();
            let agent = setup_agent();

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

            let observed = agent.generic_receive(invocation.clone(), now.into())?;
            pretty::assert_eq!(observed, Recipient::Other(invocation.payload));
            Ok(())
        }

        #[test_log::test]
        fn test_expired() -> TestResult {
            let (past, now, _exp) = setup_valid_time();
            let agent = setup_agent();

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
                Some(past.try_into()?),
                Some(now.try_into()?),
                now.into(),
                header::EdDsaHeader {
                    codec: encoding::Preset::DagCbor,
                }
                .into(),
            )?;

            let observed = agent.generic_receive(invocation.clone(), now.into());
            pretty::assert_eq!(
                observed
                    .unwrap_err()
                    .as_validation_error()
                    .ok_or("not a validation error")?,
                &ValidationError::Expired
            );
            Ok(())
        }

        #[test_log::test]
        fn test_invalid_sig() -> TestResult {
            let (_past, now, _exp) = setup_valid_time();
            let agent = setup_agent();
            let server = &agent.did;

            let mut invocation = agent.invoke(
                None,
                agent.did.clone(),
                Read {
                    path: None,
                    args: None,
                }
                .into(),
                BTreeMap::new(),
                None,
                None,
                Some(now.try_into()?),
                now.into(),
                header::EdDsaHeader {
                    codec: encoding::Preset::DagCbor,
                }
                .into(),
            )?;

            let (not_server, _) = gen_did();

            invocation.payload.issuer = not_server.clone();
            invocation.payload.audience = Some(server.clone());
            invocation.payload.subject = not_server;

            let observed = agent.generic_receive(invocation, now.into());

            assert_matches!(
                observed,
                Err(ReceiveError::SigVerifyError(
                    crate::crypto::signature::ValidateError::VerifyError(_)
                ))
            );

            Ok(())
        }
    }

    mod chain {
        use super::*;
        use assert_matches::assert_matches;

        struct Ctx {
            varsig_header: crate::crypto::varsig::header::Preset,
            dnslink_len: usize,
            inv_store: crate::invocation::store::MemoryStore<AccountManage>,
            del_store: crate::delegation::store::MemoryStore,
            account_invocation: Invocation<AccountManage>,
            server: crate::did::preset::Verifier,
            server_signer: crate::did::preset::Signer,
            device: crate::did::preset::Verifier,
            dnslink: crate::did::preset::Verifier,
        }

        fn setup_test_chain() -> Result<Ctx, Box<dyn std::error::Error>> {
            let (server, server_signer) = gen_did();
            let (account, account_signer) = gen_did();
            let (device, device_signer) = gen_did();
            let (dnslink, dnslink_signer) = gen_did();

            let varsig_header = crate::crypto::varsig::header::Preset::EdDsa(
                crate::crypto::varsig::header::EdDsaHeader {
                    codec: crate::crypto::varsig::encoding::Preset::DagCbor,
                },
            );

            let inv_store = crate::invocation::store::MemoryStore::default();
            let del_store = crate::delegation::store::MemoryStore::default();

            // Scenario
            // ========
            //
            // Delegations
            // 1.               account -*-> server
            // 2.                            server -a-> device
            // 3.  dnslink -d-> account
            //
            // Invocation
            // 4. [dnslink -d-> account -*-> server -a-> device]

            // 1.               account -*-> server
            let account_to_server = crate::Delegation::try_sign(
                &account_signer,
                varsig_header.clone(),
                crate::delegation::PayloadBuilder::default()
                    .subject(None)
                    .issuer(account.clone())
                    .audience(server.clone())
                    .command("/".into())
                    .expiration(crate::time::Timestamp::five_years_from_now())
                    .build()?,
            )?;

            // 2.                            server -a-> device
            let server_to_device = crate::Delegation::try_sign(
                &server_signer,
                varsig_header.clone(), // FIXME can also put this on a builder
                crate::delegation::PayloadBuilder::default()
                    .subject(None) // FIXME needs a sibject when we figure out powerbox
                    .issuer(server.clone())
                    .audience(device.clone())
                    .command("/".into())
                    .expiration(crate::time::Timestamp::five_years_from_now())
                    .build()?, // I don't love this is now failable
            )?;

            // 3.  dnslink -d-> account
            let dnslink_to_account = crate::Delegation::try_sign(
                &dnslink_signer,
                varsig_header.clone(),
                crate::delegation::PayloadBuilder::default()
                    .subject(Some(dnslink.clone()))
                    .issuer(dnslink.clone())
                    .audience(account.clone())
                    .command("/".into())
                    .expiration(crate::time::Timestamp::five_years_from_now())
                    .build()?,
            )?;

            del_store.insert(account_to_server.clone())?;
            del_store.insert(server_to_device.clone())?;
            del_store.insert(dnslink_to_account.clone())?;

            let chain_for_dnslink: Vec<Cid> = del_store
                .get_chain(
                    &device,
                    &dnslink.clone(),
                    "/".into(),
                    vec![],
                    SystemTime::now(),
                )?
                .ok_or("failed during proof lookup")?
                .iter()
                .map(|x| x.0.clone())
                .collect();

            // 4. [dnslink -d-> account -*-> server -a-> device]
            let account_invocation = crate::Invocation::try_sign(
                &device_signer,
                varsig_header.clone(),
                crate::invocation::PayloadBuilder::default()
                    .subject(dnslink.clone())
                    .issuer(device.clone())
                    .audience(Some(server.clone()))
                    .ability(AccountManage)
                    .proofs(chain_for_dnslink.clone())
                    .build()?,
            )?;

            let dnslink_len = chain_for_dnslink.len();

            Ok(Ctx {
                varsig_header,
                dnslink_len,
                inv_store,
                del_store,
                account_invocation,
                server,
                server_signer,
                device,
                dnslink,
            })
        }

        #[test_log::test]
        fn test_chain_ok() -> TestResult {
            let ctx = setup_test_chain()?;

            let agent = Agent::new(
                ctx.server.clone(),
                ctx.server_signer.clone(),
                &ctx.inv_store,
                &ctx.del_store,
            );

            let observed = agent.receive(ctx.account_invocation.clone());
            assert_matches!(observed, Ok(Recipient::You(_)));
            Ok(())
        }

        #[test_log::test]
        fn test_chain_wrong_sub() -> TestResult {
            let ctx = setup_test_chain()?;

            let agent = Agent::new(
                ctx.server.clone(),
                ctx.server_signer.clone(),
                &ctx.inv_store,
                &ctx.del_store,
            );

            let not_account_invocation = crate::Invocation::try_sign(
                &ctx.server_signer,
                ctx.varsig_header,
                crate::invocation::PayloadBuilder::default()
                    .subject(ctx.dnslink.clone())
                    .issuer(ctx.server.clone())
                    .audience(Some(ctx.device.clone()))
                    .ability(AccountManage)
                    .proofs(vec![]) // FIXME
                    .build()?,
            )?;

            let observed_other = agent.receive(not_account_invocation);
            assert_matches!(
                observed_other,
                Err(ReceiveError::ValidationError(
                    ValidationError::DidNotTerminateInSubject
                ))
            );

            Ok(())
        }
    }
}
