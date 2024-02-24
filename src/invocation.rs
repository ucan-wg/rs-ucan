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
pub use payload::*;

use crate::{
    ability,
    crypto::{signature, varsig},
    did::{self, Did},
    time::{Expired, Timestamp},
};
use libipld_core::{
    cid::Cid,
    codec::{Codec, Encode},
    ipld::Ipld,
};
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
pub struct Invocation<
    A,
    DID: did::Did,
    V: varsig::Header<Enc>,
    Enc: Codec + TryFrom<u32> + Into<u32>,
>(pub signature::Envelope<payload::Payload<A, DID>, DID, V, Enc>);

/// A variant of [`Invocation`] that has the abilties and DIDs from this library pre-filled.
pub type Preset = Invocation<
    ability::preset::Ready,
    did::preset::Verifier,
    varsig::header::Preset,
    varsig::encoding::Preset,
>;

pub type PresetPromised = Invocation<
    ability::preset::Promised,
    did::preset::Verifier,
    varsig::header::Preset,
    varsig::encoding::Preset,
>;

impl<A, DID: Did, V: varsig::Header<Enc>, Enc: Codec + TryFrom<u32> + Into<u32>>
    Invocation<A, DID, V, Enc>
where
    Ipld: Encode<Enc>,
{
    pub fn new(payload: Payload<A, DID>, varsig_header: V, signature: DID::Signature) -> Self {
        Invocation(signature::Envelope::new(varsig_header, signature, payload))
    }

    pub fn payload(&self) -> &Payload<A, DID> {
        &self.0.payload
    }

    pub fn varsig_header(&self) -> &V {
        &self.0.varsig_header
    }

    pub fn signature(&self) -> &DID::Signature {
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

    pub fn map_ability<F, Z>(self, f: F) -> Invocation<Z, DID, V, Enc>
    where
        F: FnOnce(A) -> Z,
    {
        Invocation(signature::Envelope::new(
            self.0.varsig_header,
            self.0.signature,
            self.0.payload.map_ability(f),
        ))
    }

    pub fn proofs(&self) -> &Vec<Cid> {
        &self.payload().proofs
    }

    pub fn issued_at(&self) -> &Option<Timestamp> {
        &self.payload().issued_at
    }

    pub fn expiration(&self) -> &Option<Timestamp> {
        &self.payload().expiration
    }

    pub fn check_time(&self, now: SystemTime) -> Result<(), Expired> {
        self.payload().check_time(now)
    }

    pub fn codec(&self) -> &Enc {
        self.varsig_header().codec()
    }

    pub fn cid(&self) -> Result<Cid, libipld_core::error::Error>
    where
        signature::Envelope<Payload<A, DID>, DID, V, Enc>: Clone,
        Ipld: Encode<Enc>,
    {
        self.0.cid()
    }

    pub fn try_sign(
        signer: &DID::Signer,
        varsig_header: V,
        payload: Payload<A, DID>,
    ) -> Result<Invocation<A, DID, V, Enc>, signature::SignError>
    where
        Payload<A, DID>: Clone,
    {
        let envelope = signature::Envelope::try_sign(signer, varsig_header, payload)?;
        Ok(Invocation(envelope))
    }

    pub fn validate_signature(&self) -> Result<(), signature::ValidateError>
    where
        Payload<A, DID>: Clone,
    {
        self.0.validate_signature()
    }
}

impl<A, DID: Did, V: varsig::Header<Enc>, Enc: Codec + TryFrom<u32> + Into<u32>>
    did::Verifiable<DID> for Invocation<A, DID, V, Enc>
{
    fn verifier(&self) -> &DID {
        &self.0.verifier()
    }
}

impl<T, DID: Did, V: varsig::Header<Enc>, Enc: Codec + TryFrom<u32> + Into<u32>>
    Invocation<T, DID, V, Enc>
{
}

impl<T, DID: Did, V: varsig::Header<Enc>, Enc: Codec + TryFrom<u32> + Into<u32>>
    From<Invocation<T, DID, V, Enc>> for Ipld
{
    fn from(invocation: Invocation<T, DID, V, Enc>) -> Self {
        invocation.0.into()
    }
}
