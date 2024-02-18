//! An [`Delegation`] is the way to grant someone else the use of [`Ability`][crate::ability].
//!
//! ## Data
//!
//! - [`Delegation`] is the top-level, signed data struture.
//! - [`Payload`] is the fields unique to an invocation.
//! - [`Preset`] is an [`Delegation`] preloaded with this library's [preset abilities](crate::ability::preset::Ready).
//! - [`Condition`]s are syntactically-driven validation rules for [`Delegation`]s.
//!
//! ## Stateful Helpers
//!
//! - [`Agent`] is a high-level interface for sessions that will involve more than one invoctaion.
//! - [`store`] is an interface for caching [`Delegation`]s.

pub mod condition;
pub mod store;

mod agent;
mod delegable;
mod payload;

pub use agent::Agent;
pub use delegable::Delegable;
pub use payload::{Payload, ValidationError};

use crate::{
    ability,
    did::{self, Did},
    nonce::Nonce,
    proof::{parents::CheckParents, same::CheckSame},
    signature,
    time::{TimeBoundError, Timestamp},
};
use condition::Condition;
use libipld_core::ipld::Ipld;
use std::collections::BTreeMap;
use web_time::SystemTime;

/// A [`Delegation`] is a signed delegation [`Payload`]
///
/// A [`Payload`] on its own is not a valid [`Delegation`], as it must be signed by the issuer.
///
/// # Examples
/// FIXME
#[derive(Clone, Debug, PartialEq)]
pub struct Delegation<D, C: Condition, DID: Did>(pub signature::Envelope<Payload<D, C, DID>, DID>);

/// A variant of [`Delegation`] that has the abilties and DIDs from this library pre-filled.
pub type Preset = Delegation<ability::preset::Builder, condition::Preset, did::preset::Verifier>;

// FIXME checkable -> provable?

impl<B, C: Condition, DID: Did> Delegation<B, C, DID> {
    /// Retrive the `issuer` of a [`Delegation`]
    pub fn issuer(&self) -> &DID {
        &self.0.payload.issuer
    }

    /// Retrive the `subject` of a [`Delegation`]
    pub fn subject(&self) -> &DID {
        &self.0.payload.subject
    }

    /// Retrive the `audience` of a [`Delegation`]
    pub fn audience(&self) -> &DID {
        &self.0.payload.audience
    }

    /// Retrive the `ability_builder` of a [`Delegation`]
    pub fn ability_builder(&self) -> &B {
        &self.0.payload.ability_builder
    }

    pub fn map_ability_builder<F, T>(self, f: F) -> Delegation<T, C, DID>
    where
        F: FnOnce(B) -> T,
    {
        Delegation(signature::Envelope {
            payload: self.0.payload.map_ability(f),
            signature: self.0.signature,
        })
    }

    /// Retrive the `condition` of a [`Delegation`]
    pub fn conditions(&self) -> &[C] {
        &self.0.payload.conditions
    }

    /// Retrive the `metadata` of a [`Delegation`]
    pub fn metadata(&self) -> &BTreeMap<String, Ipld> {
        &self.0.payload.metadata
    }

    /// Retrive the `nonce` of a [`Delegation`]
    pub fn nonce(&self) -> &Nonce {
        &self.0.payload.nonce
    }

    /// Retrive the `not_before` of a [`Delegation`]
    pub fn not_before(&self) -> Option<&Timestamp> {
        self.0.payload.not_before.as_ref()
    }

    /// Retrive the `expiration` of a [`Delegation`]
    pub fn expiration(&self) -> &Timestamp {
        &self.0.payload.expiration
    }

    pub fn check_time(&self, now: SystemTime) -> Result<(), TimeBoundError> {
        self.0.payload.check_time(now)
    }

    pub fn payload(&self) -> &Payload<B, C, DID> {
        &self.0.payload
    }

    pub fn signature(&self) -> &signature::Witness<DID::Signature> {
        &self.0.signature
    }

    pub fn validate_signature(&self) -> Result<(), signature::ValidateError>
    where
        Payload<B, C, DID>: Clone,
    {
        self.0.validate_signature()
    }

    pub fn try_sign(
        signer: &DID::Signer,
        payload: Payload<B, C, DID>,
    ) -> Result<Self, signature::SignError>
    where
        Payload<B, C, DID>: Clone,
    {
        signature::Envelope::try_sign(signer, payload).map(Delegation)
    }
}

impl<B: CheckSame, C: Condition, DID: Did> CheckSame for Delegation<B, C, DID> {
    type Error = <B as CheckSame>::Error;

    fn check_same(&self, proof: &Delegation<B, C, DID>) -> Result<(), Self::Error> {
        self.0.payload.check_same(&proof.payload())
    }
}

impl<T: CheckParents, C: Condition, DID: Did> CheckParents for Delegation<T, C, DID> {
    type Parents = Delegation<T::Parents, C, DID>;
    type ParentError = <T as CheckParents>::ParentError;

    fn check_parent(&self, proof: &Self::Parents) -> Result<(), Self::ParentError> {
        self.payload().check_parent(&proof.payload())
    }
}
