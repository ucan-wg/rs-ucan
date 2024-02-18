//! An [`Invocation`] is a request to use an [`Ability`][crate::ability].
//!
//! ## Data
//!
//! - [`Invocation`] is the top-level, signed data struture.
//! - [`Payload`] is the fields unique to an invocation.
//! - [`Preset`] is an [`Invocation`] preloaded with this library's [preset abilities](crate::ability::preset::Ready).
//! - [`promise`]s are a mechanism to chain invocations together.
//!
//! ## Stateful Helpers
//!
//! - [`Agent`] is a high-level interface for sessions that will involve more than one invoctaion.
//! - [`store`] is an interface for caching [`Invocation`]s.

mod agent;
mod payload;

pub mod promise;
pub mod store;

pub use agent::Agent;
pub use payload::{Payload, Promised};

use crate::{
    ability, did,
    did::Did,
    signature,
    time::{Expired, Timestamp},
};
use libipld_core::{cid::Cid, ipld::Ipld};
use web_time::SystemTime;

/// The complete, signed [`invocation::Payload`][Payload].
///
/// Invocations are the actual "doing" in the UCAN lifecycle.
/// Unlike [`Delegation`][crate::Delegation]s, which live for some period of time and
/// can be used multiple times, [`Invocation`]s are unique and single-use.
///
/// # Expiration
///
/// `Invocations` include an optional expiration field which behaves like a timeout:
/// "if this isn't run by a the expiration time, I'm going to assume that it didn't happen."
/// This is a best practice in message-passing distributed systems because the network is
/// [unreliable](https://en.wikipedia.org/wiki/Fallacies_of_distributed_computing).
#[derive(Debug, Clone, PartialEq)]
pub struct Invocation<A, DID: did::Did>(pub signature::Envelope<payload::Payload<A, DID>, DID>);

/// A variant of [`Invocation`] that has the abilties and DIDs from this library pre-filled.
pub type Preset = Invocation<ability::preset::Ready, did::preset::Verifier>;

pub type PresetPromised = Invocation<ability::preset::Promised, did::preset::Verifier>;

impl<A, DID: Did> Invocation<A, DID> {
    pub fn new(payload: Payload<A, DID>, signature: signature::Witness<DID::Signature>) -> Self {
        Invocation(signature::Envelope { payload, signature })
    }

    pub fn payload(&self) -> &Payload<A, DID> {
        &self.0.payload
    }

    pub fn signature(&self) -> &signature::Witness<DID::Signature> {
        &self.0.signature
    }

    pub fn audience(&self) -> &Option<DID> {
        &self.0.payload.audience
    }

    pub fn issuer(&self) -> &DID {
        &self.0.payload.issuer
    }

    pub fn subject(&self) -> &DID {
        &self.0.payload.subject
    }

    pub fn ability(&self) -> &A {
        &self.0.payload.ability
    }

    pub fn map_ability<F, Z>(self, f: F) -> Invocation<Z, DID>
    where
        F: FnOnce(A) -> Z,
    {
        Invocation(signature::Envelope {
            payload: self.0.payload.map_ability(f),
            signature: self.0.signature,
        })
    }

    pub fn proofs(&self) -> &Vec<Cid> {
        &self.0.payload.proofs
    }

    pub fn issued_at(&self) -> &Option<Timestamp> {
        &self.0.payload.issued_at
    }

    pub fn expiration(&self) -> &Option<Timestamp> {
        &self.0.payload.expiration
    }

    pub fn check_time(&self, now: SystemTime) -> Result<(), Expired>
    where
        A: Clone,
    {
        self.0.payload.check_time(now)
    }

    pub fn try_sign(
        signer: &DID::Signer,
        payload: Payload<A, DID>,
    ) -> Result<Invocation<A, DID>, signature::SignError>
    where
        Payload<A, DID>: Clone,
    {
        let envelope = signature::Envelope::try_sign(signer, payload)?;
        Ok(Invocation(envelope))
    }
}

impl<A, DID: Did> did::Verifiable<DID> for Invocation<A, DID> {
    fn verifier(&self) -> &DID {
        &self.0.verifier()
    }
}

impl<T, DID: Did> Invocation<T, DID> {}

impl<T, DID: Did> From<Invocation<T, DID>> for Ipld {
    fn from(invocation: Invocation<T, DID>) -> Self {
        invocation.0.into()
    }
}
