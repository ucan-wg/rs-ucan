pub mod condition;
pub mod error;
pub mod store;

mod agent;
mod delegable;
mod payload;

pub use agent::Agent;
pub use delegable::Delegable;
pub use payload::Payload;

use crate::{
    ability,
    did::{self, Did},
    nonce::Nonce,
    proof::{checkable::Checkable, parents::CheckParents, same::CheckSame},
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
/// FIXME wrap in struct to make the docs & error messages better?
pub type Delegation<T, C, DID> = signature::Envelope<Payload<T, C, DID>, DID>;

pub type Preset = Delegation<ability::preset::Builder, condition::Preset, did::preset::Verifier>;

// FIXME checkable -> provable?

impl<B: Checkable, C: Condition, DID: Did> Delegation<B, C, DID> {
    /// Retrive the `issuer` of a [`Delegation`]
    pub fn issuer(&self) -> &DID {
        &self.payload.issuer
    }

    /// Retrive the `subject` of a [`Delegation`]
    pub fn subject(&self) -> &DID {
        &self.payload.subject
    }

    /// Retrive the `audience` of a [`Delegation`]
    pub fn audience(&self) -> &DID {
        &self.payload.audience
    }

    /// Retrive the `ability_builder` of a [`Delegation`]
    pub fn ability_builder(&self) -> &B {
        &self.payload.ability_builder
    }

    /// Retrive the `condition` of a [`Delegation`]
    pub fn conditions(&self) -> &[C] {
        &self.payload.conditions
    }

    /// Retrive the `metadata` of a [`Delegation`]
    pub fn metadata(&self) -> &BTreeMap<String, Ipld> {
        &self.payload.metadata
    }

    /// Retrive the `nonce` of a [`Delegation`]
    pub fn nonce(&self) -> &Nonce {
        &self.payload.nonce
    }

    /// Retrive the `not_before` of a [`Delegation`]
    pub fn not_before(&self) -> Option<&Timestamp> {
        self.payload.not_before.as_ref()
    }

    /// Retrive the `expiration` of a [`Delegation`]
    pub fn expiration(&self) -> &Timestamp {
        &self.payload.expiration
    }

    /// Retrive the `signature` of a [`Delegation`]
    pub fn check_time(&self, now: SystemTime) -> Result<(), TimeBoundError> {
        self.payload.check_time(now)
    }
}

impl<T: CheckSame, C: Condition, DID: Did> CheckSame for Delegation<T, C, DID> {
    type Error = <T as CheckSame>::Error;

    fn check_same(&self, proof: &Delegation<T, C, DID>) -> Result<(), Self::Error> {
        self.payload.check_same(&proof.payload)
    }
}

impl<T: CheckParents, C: Condition, DID: Did> CheckParents for Delegation<T, C, DID> {
    type Parents = Delegation<T::Parents, C, DID>;
    type ParentError = <T as CheckParents>::ParentError;

    fn check_parent(&self, proof: &Self::Parents) -> Result<(), Self::ParentError> {
        self.payload.check_parent(&proof.payload)
    }
}
