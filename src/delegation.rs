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

use crate::ability::arguments::Named;
use crate::{
    capsule::Capsule,
    crypto::{signature::Envelope, varsig, Nonce},
    did::{self, Did},
    time::{TimeBoundError, Timestamp},
};
use libipld_core::link::Link;
use libipld_core::{codec::Codec, ipld::Ipld};
use policy::Predicate;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use web_time::SystemTime;

/// A [`Delegation`] is a signed delegation [`Payload`]
///
/// A [`Payload`] on its own is not a valid [`Delegation`], as it must be signed by the issuer.
#[derive(Clone, Debug, PartialEq)]
pub struct Delegation<
    DID: Did = did::preset::Verifier,
    V: varsig::Header<C> = varsig::header::Preset,
    C: Codec + TryFrom<u64> + Into<u64> = varsig::encoding::Preset,
> {
    pub varsig_header: V,
    pub payload: Payload<DID>,
    pub signature: DID::Signature,
    _marker: std::marker::PhantomData<C>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Proof<
    DID: Did = did::preset::Verifier,
    V: varsig::Header<C> = varsig::header::Preset,
    C: Codec + TryFrom<u64> + Into<u64> = varsig::encoding::Preset,
> {
    pub prf: Vec<Link<Delegation<DID, V, C>>>,
}

impl<DID: Did, V: varsig::Header<C>, C: Codec + TryFrom<u64> + Into<u64>> Capsule
    for Proof<DID, V, C>
{
    const TAG: &'static str = "ucan/prf";
}

impl<DID: Did, V: varsig::Header<C>, C: Codec + Into<u64> + TryFrom<u64>> Delegation<DID, V, C> {
    pub fn new(
        varsig_header: V,
        signature: DID::Signature,
        payload: Payload<DID>,
    ) -> Delegation<DID, V, C> {
        Delegation {
            varsig_header,
            payload,
            signature,
            _marker: std::marker::PhantomData,
        }
    }

    /// Retrive the `issuer` of a [`Delegation`]
    pub fn issuer(&self) -> &DID {
        &self.payload.issuer
    }

    /// Retrive the `subject` of a [`Delegation`]
    pub fn subject(&self) -> &Option<DID> {
        &self.payload.subject
    }

    /// Retrive the `audience` of a [`Delegation`]
    pub fn audience(&self) -> &DID {
        &self.payload.audience
    }

    /// Retrive the `policy` of a [`Delegation`]
    pub fn policy(&self) -> &Vec<Predicate> {
        &self.payload.policy
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

    pub fn check_time(&self, now: SystemTime) -> Result<(), TimeBoundError> {
        self.payload.check_time(now)
    }
}

impl<DID: Did + Clone, V: varsig::Header<C> + Clone, C: Codec + TryFrom<u64> + Into<u64>> Envelope
    for Delegation<DID, V, C>
where
    Payload<DID>: TryFrom<Named<Ipld>>,
    Named<Ipld>: From<Payload<DID>>,
{
    type DID = DID;
    type Payload = Payload<DID>;
    type VarsigHeader = V;
    type Encoder = C;

    fn construct(
        varsig_header: V,
        signature: DID::Signature,
        payload: Payload<DID>,
    ) -> Delegation<DID, V, C> {
        Delegation {
            varsig_header,
            payload,
            signature,
            _marker: std::marker::PhantomData,
        }
    }

    fn varsig_header(&self) -> &V {
        &self.varsig_header
    }

    fn payload(&self) -> &Payload<DID> {
        &self.payload
    }

    fn signature(&self) -> &DID::Signature {
        &self.signature
    }

    fn verifier(&self) -> &DID {
        &self.payload.issuer
    }
}

impl<DID: Did + Clone, V: varsig::Header<C> + Clone, C: Codec + TryFrom<u64> + Into<u64>> Serialize
    for Delegation<DID, V, C>
where
    Payload<DID>: TryFrom<Named<Ipld>>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_ipld_envelope().serialize(serializer)
    }
}

impl<'de, DID: Did + Clone, V: varsig::Header<C> + Clone, C: Codec + TryFrom<u64> + Into<u64>>
    Deserialize<'de> for Delegation<DID, V, C>
where
    Payload<DID>: TryFrom<Named<Ipld>>,
    <Payload<DID> as TryFrom<Named<Ipld>>>::Error: std::fmt::Display,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let ipld = Ipld::deserialize(deserializer)?;
        Self::try_from_ipld_envelope(ipld).map_err(serde::de::Error::custom)
    }
}
