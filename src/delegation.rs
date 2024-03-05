//! A [`Delegation`] is the way to grant someone else the use of [`Ability`][crate::ability].
//!
//! ## Data
//!
//! - [`Delegation`] is the top-level, signed data struture.
//! - [`Payload`] is the fields unique to an invocation.
//! - [`Preset`] is an [`Delegation`] preloaded with this library's [preset abilities](crate::ability::preset::Ready).
//! - [`Predicate`]s are syntactically-driven validation rules for [`Delegation`]s.
//!
//! ## Stateful Helpers
//!
//! - [`Agent`] is a high-level interface for sessions that will involve more than one invoctaion.
//! - [`store`] is an interface for caching [`Delegation`]s.

pub mod policy;
pub mod store;

mod agent;
mod payload;

pub use agent::Agent;
pub use payload::*;

use crate::{
    capsule::Capsule,
    crypto::{signature, varsig, Nonce},
    did::{self, Did},
    time::{TimeBoundError, Timestamp},
};
use libipld_core::{
    cid::Cid,
    codec::{Codec, Encode},
    ipld::Ipld,
};
use policy::Predicate;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use web_time::SystemTime;

/// A [`Delegation`] is a signed delegation [`Payload`]
///
/// A [`Payload`] on its own is not a valid [`Delegation`], as it must be signed by the issuer.
///
/// # Examples
/// FIXME
#[derive(Clone, Debug, PartialEq)]
pub struct Delegation<DID: Did, V: varsig::Header<Enc>, Enc: Codec + TryFrom<u32> + Into<u32>>(
    pub signature::Envelope<Payload<DID>, DID, V, Enc>,
);

#[derive(Clone, Debug, PartialEq)]
pub struct Chain<DID: Did, V: varsig::Header<Enc>, Enc: Codec + TryFrom<u32> + Into<u32>>(
    Vec<Delegation<DID, V, Enc>>,
);

impl<DID: Did, V: varsig::Header<Enc>, Enc: Codec + TryFrom<u32> + Into<u32>> Capsule
    for Chain<DID, V, Enc>
{
    const TAG: &'static str = "ucan/chain";
}

/// A variant of [`Delegation`] that has the abilties and DIDs from this library pre-filled.
pub type Preset =
    Delegation<did::preset::Verifier, varsig::header::Preset, varsig::encoding::Preset>;

// FIXME checkable -> provable?

impl<DID: Did, V: varsig::Header<Enc>, Enc: Codec + Into<u32> + TryFrom<u32>>
    Delegation<DID, V, Enc>
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

    /// Retrive the `policy` of a [`Delegation`]
    pub fn policy(&self) -> &Vec<Predicate> {
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

    pub fn payload(&self) -> &Payload<DID> {
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
        signature::Envelope<Payload<DID>, DID, V, Enc>: Clone + Encode<Enc>,
        Ipld: Encode<Enc>,
    {
        self.0.cid()
    }

    pub fn validate_signature(&self) -> Result<(), signature::ValidateError>
    where
        Payload<DID>: Clone,
        Ipld: Encode<Enc>,
    {
        self.0.validate_signature()
    }

    pub fn try_sign(
        signer: &DID::Signer,
        varsig_header: V,
        payload: Payload<DID>,
    ) -> Result<Self, signature::SignError>
    where
        Ipld: Encode<Enc>,
        Payload<DID>: Clone,
    {
        signature::Envelope::try_sign(signer, varsig_header, payload).map(Delegation)
    }
}

impl<DID: Did, V: varsig::Header<Enc>, Enc: Codec + TryFrom<u32> + Into<u32>> TryFrom<Ipld>
    for Delegation<DID, V, Enc>
where
    Payload<DID>: TryFrom<Ipld>,
{
    type Error = <signature::Envelope<Payload<DID>, DID, V, Enc> as TryFrom<Ipld>>::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        signature::Envelope::try_from(ipld).map(Delegation)
    }
}

impl<DID: Did, V: varsig::Header<Enc>, Enc: Codec + TryFrom<u32> + Into<u32>>
    From<Delegation<DID, V, Enc>> for Ipld
{
    fn from(delegation: Delegation<DID, V, Enc>) -> Self {
        delegation.0.into()
    }
}

impl<DID: Did, V: varsig::Header<Enc>, Enc: Codec + TryFrom<u32> + Into<u32>> Serialize
    for Delegation<DID, V, Enc>
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, DID: Did, V: varsig::Header<Enc>, Enc: Codec + TryFrom<u32> + Into<u32>> Deserialize<'de>
    for Delegation<DID, V, Enc>
where
    Payload<DID>: TryFrom<Ipld>,
    <Payload<DID> as TryFrom<Ipld>>::Error: std::fmt::Display,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        signature::Envelope::deserialize(deserializer).map(Delegation)
    }
}
