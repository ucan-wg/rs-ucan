use super::{condition::Condition, payload::Payload, store::Store, Delegation};
use crate::{did::Did, nonce::Nonce, proof::checkable::Checkable, time::JsTime};
use libipld_core::{cid::Cid, ipld::Ipld};
use std::{collections::BTreeMap, marker::PhantomData};
use thiserror::Error;
use web_time::SystemTime;

pub struct Agent<'a, B: Checkable, C: Condition, DID: Did, S: Store<B, C, DID>> {
    pub did: &'a DID,
    pub store: &'a mut S,

    signer: &'a <DID as Did>::Signer,
    _marker: PhantomData<(B, C)>,
}

// FIXME show example of multiple hierarchies of "all things accepted"
// delegating down to inner versions of this

impl<
        'a,
        B: Checkable + Clone,
        C: Condition + Clone,
        DID: Did + ToString + Clone,
        S: Store<B, C, DID>,
    > Agent<'a, B, C, DID, S>
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
        expiration: JsTime,
        not_before: Option<JsTime>,
    ) -> Result<Delegation<B, C, DID>, DelegateError<<S as Store<B, C, DID>>::Error>> {
        let mut salt = self.did.clone().to_string().into_bytes();
        let nonce = Nonce::generate_16(&mut salt);

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

            return Ok(Delegation::try_sign::<DID>(self.signer, payload).expect("FIXME"));
        }

        let to_delegate = &self
            .store
            .get_chain(&self.did, &subject, &ability_builder, &SystemTime::now())
            .map_err(DelegateError::StoreError)?
            .ok_or(DelegateError::ProofsNotFound)?
            .first()
            .1
            .payload;

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

        Ok(Delegation::try_sign::<DID>(self.signer, payload).expect("FIXME"))
    }

    pub fn recieve(
        &mut self,
        cid: Cid, // FIXME remove and generate from the capsule header?
        delegation: Delegation<B, C, DID>,
    ) -> Result<(), ReceiveError<<S as Store<B, C, DID>>::Error, DID>> {
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
