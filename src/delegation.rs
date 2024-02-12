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
    did::Did,
    nonce::Nonce,
    proof::{checkable::Checkable, parents::CheckParents, same::CheckSame},
    signature,
    time::{TimeBoundError, Timestamp},
};
use condition::Condition;
use libipld_core::ipld::Ipld;
use std::{collections::BTreeMap, convert::AsRef};
use web_time::SystemTime;

/// A [`Delegation`] is a signed delegation [`Payload`]
///
/// A [`Payload`] on its own is not a valid [`Delegation`], as it must be signed by the issuer.
///
/// # Examples
/// FIXME
pub type Delegation<T, C> = signature::Envelope<Payload<T, C>>;

// FIXME checkable -> provable?

impl<B: Checkable, C: Condition> Delegation<B, C> {
    pub fn issuer(&self) -> &Did {
        &self.payload.issuer
    }

    pub fn subject(&self) -> &Did {
        &self.payload.subject
    }

    pub fn audience(&self) -> &Did {
        &self.payload.audience
    }

    pub fn ability_builder(&self) -> &B {
        &self.payload.ability_builder
    }

    pub fn conditions(&self) -> &[C] {
        &self.payload.conditions
    }

    pub fn metadata(&self) -> &BTreeMap<String, Ipld> {
        &self.payload.metadata
    }

    pub fn nonce(&self) -> &Nonce {
        &self.payload.nonce
    }

    pub fn not_before(&self) -> Option<&Timestamp> {
        self.payload.not_before.as_ref()
    }

    pub fn expiration(&self) -> &Timestamp {
        &self.payload.expiration
    }

    pub fn check_time(&self, now: SystemTime) -> Result<(), TimeBoundError> {
        self.payload.check_time(now)
    }
}

impl<T: CheckSame, C: Condition> CheckSame for Delegation<T, C> {
    type Error = <T as CheckSame>::Error;

    fn check_same(&self, proof: &Delegation<T, C>) -> Result<(), Self::Error> {
        self.payload.check_same(&proof.payload)
    }
}

impl<T: CheckParents, C: Condition> CheckParents for Delegation<T, C> {
    type Parents = Delegation<T::Parents, C>;
    type ParentError = <T as CheckParents>::ParentError;

    fn check_parent(&self, proof: &Self::Parents) -> Result<(), Self::ParentError> {
        self.payload.check_parent(&proof.payload)
    }
}
