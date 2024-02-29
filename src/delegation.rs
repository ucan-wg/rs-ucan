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
pub mod policy;
pub mod store;

mod agent;
mod payload;

pub use agent::Agent;
pub use payload::*;

use crate::capsule::Capsule;
use crate::{
    // ability,
    crypto::{signature, varsig, Nonce},
    did::{self, Did},
    time::{TimeBoundError, Timestamp},
};
use condition::Condition;
use libipld_core::{
    cid::Cid,
    codec::{Codec, Encode},
    ipld::Ipld,
};
use std::collections::BTreeMap;
use web_time::SystemTime;

/// A [`Delegation`] is a signed delegation [`Payload`]
///
/// A [`Payload`] on its own is not a valid [`Delegation`], as it must be signed by the issuer.
///
/// # Examples
/// FIXME
#[derive(Clone, Debug, PartialEq)]
pub struct Delegation<
    C: Condition,
    DID: Did,
    V: varsig::Header<Enc>,
    Enc: Codec + TryFrom<u32> + Into<u32>,
>(pub signature::Envelope<Payload<C, DID>, DID, V, Enc>);

#[derive(Clone, Debug, PartialEq)]
pub struct Chain<
    C: Condition,
    DID: Did,
    V: varsig::Header<Enc>,
    Enc: Codec + TryFrom<u32> + Into<u32>,
>(Vec<Delegation<C, DID, V, Enc>>);

impl<C: Condition, DID: Did, V: varsig::Header<Enc>, Enc: Codec + TryFrom<u32> + Into<u32>> Capsule
    for Chain<C, DID, V, Enc>
{
    const TAG: &'static str = "ucan/chain";
}

/// A variant of [`Delegation`] that has the abilties and DIDs from this library pre-filled.
pub type Preset = Delegation<
    condition::Preset,
    did::preset::Verifier,
    varsig::header::Preset,
    varsig::encoding::Preset,
>;

// FIXME checkable -> provable?

impl<C: Condition, DID: Did, V: varsig::Header<Enc>, Enc: Codec + Into<u32> + TryFrom<u32>>
    Delegation<C, DID, V, Enc>
{
    /// Retrive the `issuer` of a [`Delegation`]
    pub fn issuer(&self) -> &DID {
        &self.0.payload.issuer
    }

    /// Retrive the `subject` of a [`Delegation`]
    pub fn subject(&self) -> &Option<DID> {
        &self.0.payload.subject
    }

    /// Retrive the `audience` of a [`Delegation`]
    pub fn audience(&self) -> &DID {
        &self.0.payload.audience
    }

    /// Retrive the `condition` of a [`Delegation`]
    pub fn policy(&self) -> &[C] {
        &self.0.payload.policy
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

    pub fn payload(&self) -> &Payload<C, DID> {
        &self.0.payload
    }

    pub fn varsig_header(&self) -> &V {
        &self.0.varsig_header
    }

    pub fn varsig_encode(self, w: &mut Vec<u8>) -> Result<(), libipld_core::error::Error>
    where
        Ipld: Encode<Enc>,
    {
        self.0.varsig_encode(w)
    }

    pub fn signature(&self) -> &DID::Signature {
        &self.0.signature
    }

    pub fn codec(&self) -> &Enc {
        self.varsig_header().codec()
    }

    pub fn cid(&self) -> Result<Cid, libipld_core::error::Error>
    where
        signature::Envelope<Payload<C, DID>, DID, V, Enc>: Clone + Encode<Enc>,
        Ipld: Encode<Enc>,
    {
        self.0.cid()
    }

    pub fn validate_signature(&self) -> Result<(), signature::ValidateError>
    where
        Payload<C, DID>: Clone,
        Ipld: Encode<Enc>,
    {
        self.0.validate_signature()
    }

    pub fn try_sign(
        signer: &DID::Signer,
        varsig_header: V,
        payload: Payload<C, DID>,
    ) -> Result<Self, signature::SignError>
    where
        Ipld: Encode<Enc>,
        Payload<C, DID>: Clone,
    {
        signature::Envelope::try_sign(signer, varsig_header, payload).map(Delegation)
    }
}
