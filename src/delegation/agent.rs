use super::{condition::Condition, payload::Payload, store::Store, Delegation};
use crate::{
    crypto::{varsig, Nonce},
    did::Did,
    proof::checkable::Checkable,
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
    B: Checkable,
    C: Condition,
    DID: Did,
    S: Store<B, C, DID, V, Enc>,
    V: varsig::Header<Enc>,
    Enc: Codec + TryFrom<u32> + Into<u32>,
> {
    /// The [`Did`][Did] of the agent.
    pub did: &'a DID,

    /// The attached [`deleagtion::Store`][super::store::Store].
    pub store: &'a mut S,

    signer: &'a <DID as Did>::Signer,
    _marker: PhantomData<(B, C, V, Enc)>,
}

// FIXME show example of multiple hierarchies of "all things accepted"
// delegating down to inner versions of this

impl<
        'a,
        B: Checkable + Clone,
        C: Condition + Clone,
        DID: Did + ToString + Clone,
        S: Store<B, C, DID, V, Enc> + Clone,
        V: varsig::Header<Enc>,
        Enc: Codec + TryFrom<u32> + Into<u32>,
    > Agent<'a, B, C, DID, S, V, Enc>
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
        subject: DID,
        ability_builder: B,
        new_conditions: Vec<C>,
        metadata: BTreeMap<String, Ipld>,
        expiration: Timestamp,
        not_before: Option<Timestamp>,
        now: SystemTime,
        varsig_header: V,
    ) -> Result<Delegation<B, C, DID, V, Enc>, DelegateError<S::DelegationStoreError>> {
        let mut salt = self.did.clone().to_string().into_bytes();
        let nonce = Nonce::generate_12(&mut salt);

        if subject == *self.did {
            let payload: Payload<B, C, DID> = Payload {
                issuer: self.did.clone(),
                audience,
                subject,
                ability_builder,
                metadata,
                nonce,
                expiration: expiration.into(),
                not_before: not_before.map(Into::into),
                conditions: new_conditions,
            };

            return Ok(Delegation::try_sign(self.signer, varsig_header, payload).expect("FIXME"));
        }

        let to_delegate = &self
            .store
            .get_chain(&self.did, &subject, &ability_builder, vec![], now)
            .map_err(DelegateError::StoreError)?
            .ok_or(DelegateError::ProofsNotFound)?
            .first()
            .1
            .payload();

        let mut conditions = to_delegate.conditions.clone();
        conditions.append(&mut new_conditions.clone());

        let payload: Payload<B, C, DID> = Payload {
            issuer: self.did.clone(),
            audience,
            subject,
            ability_builder,
            conditions,
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
        delegation: Delegation<B, C, DID, V, Enc>,
    ) -> Result<(), ReceiveError<S::DelegationStoreError, DID>> {
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
