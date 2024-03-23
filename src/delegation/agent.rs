use super::{payload::Payload, policy::Predicate, store::Store, Delegation};
use crate::ability::arguments::Named;
use crate::did;
use crate::{
    crypto::{signature::Envelope, varsig, Nonce},
    did::Did,
    time::Timestamp,
};
use libipld_core::{
    cid::Cid,
    codec::{Codec, Encode},
    ipld::Ipld,
};
use std::{collections::BTreeMap, marker::PhantomData};
use thiserror::Error;
use web_time::SystemTime;

/// A stateful agent capable of delegating to others, and being delegated to.
///
/// This is helpful for sessions where more than one delegation will be made.
#[derive(Debug)]
pub struct Agent<
    S: Store<DID, V, C>,
    DID: Did + Clone = did::preset::Verifier,
    V: varsig::Header<C> + Clone = varsig::header::Preset,
    C: Codec + Into<u64> + TryFrom<u64> = varsig::encoding::Preset,
> where
    Ipld: Encode<C>,
    Payload<DID>: TryFrom<Named<Ipld>>,
    Named<Ipld>: From<Payload<DID>>,
{
    /// The [`Did`][Did] of the agent.
    pub did: DID,

    /// The attached [`deleagtion::Store`][super::store::Store].
    pub store: S,

    signer: <DID as Did>::Signer,
    _marker: PhantomData<(V, C)>,
}

impl<
        S: Store<DID, V, C> + Clone,
        DID: Did + Clone,
        V: varsig::Header<C> + Clone,
        C: Codec + TryFrom<u64> + Into<u64>,
    > Agent<S, DID, V, C>
where
    Ipld: Encode<C>,
    Payload<DID>: TryFrom<Named<Ipld>>,
    Named<Ipld>: From<Payload<DID>>,
{
    pub fn new(did: DID, signer: <DID as Did>::Signer, store: S) -> Self {
        Self {
            did,
            store,
            signer,
            _marker: PhantomData,
        }
    }

    pub fn delegate(
        &self,
        audience: DID,
        subject: Option<DID>,
        via: Option<DID>,
        command: String,
        new_policy: Vec<Predicate>,
        metadata: BTreeMap<String, Ipld>,
        expiration: Timestamp,
        not_before: Option<Timestamp>,
        now: SystemTime,
        varsig_header: V,
    ) -> Result<Delegation<DID, V, C>, DelegateError<S::DelegationStoreError>> {
        let mut salt = self.did.clone().to_string().into_bytes();
        let nonce = Nonce::generate_12(&mut salt);

        if let Some(ref sub) = subject {
            if sub == &self.did {
                let payload: Payload<DID> = Payload {
                    issuer: self.did.clone(),
                    audience,
                    subject,
                    via,
                    command,
                    metadata,
                    nonce,
                    expiration: expiration.into(),
                    not_before: not_before.map(Into::into),
                    policy: new_policy,
                };

                return Ok(
                    Delegation::try_sign(&self.signer, varsig_header, payload).expect("FIXME")
                );
            }
        }

        let proofs = &self
            .store
            .get_chain(&self.did, &subject, "/".into(), vec![], now)
            .map_err(DelegateError::StoreError)?
            .ok_or(DelegateError::ProofsNotFound)?;
        let to_delegate = proofs.first().1.payload();

        let mut policy = to_delegate.policy.clone();
        policy.append(&mut new_policy.clone());

        let payload: Payload<DID> = Payload {
            issuer: self.did.clone(),
            audience,
            subject,
            via,
            command,
            policy,
            metadata,
            nonce,
            expiration: expiration.into(),
            not_before: not_before.map(Into::into),
        };

        Ok(Delegation::try_sign(&self.signer, varsig_header, payload).expect("FIXME"))
    }

    pub fn receive(
        &self,
        cid: Cid, // FIXME remove and generate from the capsule header?
        delegation: Delegation<DID, V, C>,
    ) -> Result<(), ReceiveError<S::DelegationStoreError, DID>> {
        if self.store.get(&cid).is_ok() {
            return Ok(());
        }

        if delegation.audience() != &self.did {
            return Err(ReceiveError::WrongAudience(delegation.audience().clone()));
        }

        delegation
            .validate_signature()
            .map_err(|_| ReceiveError::InvalidSignature(cid))?;

        self.store.insert_keyed(cid, delegation).map_err(Into::into)
    }
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Error)]
pub enum DelegateError<StoreErr> {
    #[error("The current agent does not have the necessary proofs to delegate.")]
    ProofsNotFound,

    #[error(transparent)]
    StoreError(#[from] StoreErr),
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Error)]
pub enum ReceiveError<StoreErr, DID> {
    #[error("The current agent ({0}) is not the intended audience of the delegation.")]
    WrongAudience(DID),

    #[error("Signature for UCAN with CID {0} is invalid.")]
    InvalidSignature(Cid),

    #[error(transparent)]
    StoreError(#[from] StoreErr),
}
