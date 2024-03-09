use super::{payload::Payload, policy::Predicate, store::Store, Delegation};
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

/// A stateful agent capable of delegatint to others, and being delegated to.
///
/// This is helpful for sessions where more than one delegation will be made.
#[derive(Debug)]
pub struct Agent<
    'a,
    DID: Did,
    S: Store<DID, V, Enc>,
    V: varsig::Header<Enc>,
    Enc: Codec + TryFrom<u64> + Into<u64>,
> {
    /// The [`Did`][Did] of the agent.
    pub did: &'a DID,

    /// The attached [`deleagtion::Store`][super::store::Store].
    pub store: &'a mut S,

    signer: &'a <DID as Did>::Signer,
    _marker: PhantomData<(V, Enc)>,
}

impl<
        'a,
        DID: Did + Clone,
        S: Store<DID, V, Enc> + Clone,
        V: varsig::Header<Enc> + Clone,
        Enc: Codec + TryFrom<u64> + Into<u64>,
    > Agent<'a, DID, S, V, Enc>
where
    Ipld: Encode<Enc>,
{
    pub fn new(did: &'a DID, signer: &'a <DID as Did>::Signer, store: &'a mut S) -> Self {
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
        command: String,
        new_policy: Vec<Predicate>,
        metadata: BTreeMap<String, Ipld>,
        expiration: Timestamp,
        not_before: Option<Timestamp>,
        now: SystemTime,
        varsig_header: V,
    ) -> Result<Delegation<DID, V, Enc>, DelegateError<S::DelegationStoreError>>
    where
        Payload<DID>: TryFrom<Ipld>,
    {
        let mut salt = self.did.clone().to_string().into_bytes();
        let nonce = Nonce::generate_12(&mut salt);

        if let Some(ref sub) = subject {
            if sub == self.did {
                let payload: Payload<DID> = Payload {
                    issuer: self.did.clone(),
                    audience,
                    subject,
                    command,
                    metadata,
                    nonce,
                    expiration: expiration.into(),
                    not_before: not_before.map(Into::into),
                    policy: new_policy,
                };

                return Ok(
                    Delegation::try_sign(self.signer, varsig_header, payload).expect("FIXME")
                );
            }
        }

        let to_delegate = &self
            .store
            .get_chain(&self.did, &subject, "/".into(), vec![], now)
            .map_err(DelegateError::StoreError)?
            .ok_or(DelegateError::ProofsNotFound)?
            .first()
            .1
            .payload();

        let mut policy = to_delegate.policy.clone();
        policy.append(&mut new_policy.clone());

        let payload: Payload<DID> = Payload {
            issuer: self.did.clone(),
            audience,
            subject,
            command,
            policy,
            metadata,
            nonce,
            expiration: expiration.into(),
            not_before: not_before.map(Into::into),
        };

        Ok(Delegation::try_sign(self.signer, varsig_header, payload).expect("FIXME"))
    }

    pub fn receive(
        &mut self,
        cid: Cid, // FIXME remove and generate from the capsule header?
        delegation: Delegation<DID, V, Enc>,
    ) -> Result<(), ReceiveError<S::DelegationStoreError, DID>>
    where
        Payload<DID>: TryFrom<Ipld>,
    {
        if self.store.get(&cid).is_ok() {
            return Ok(());
        }

        if delegation.audience() != self.did {
            return Err(ReceiveError::WrongAudience(delegation.audience().clone()));
        }

        delegation
            .validate_signature()
            .map_err(|_| ReceiveError::InvalidSignature(cid))?;

        self.store.insert(cid, delegation).map_err(Into::into)
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
