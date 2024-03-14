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

use crate::ability::arguments::Named;
use crate::ability::command::ToCommand;
use crate::{
    crypto::{signature::Envelope, varsig},
    did::{self, Did},
    time::{Expired, Timestamp},
};
use libipld_core::{
    cid::Cid,
    codec::{Codec, Encode},
    ipld::Ipld,
};
use serde::{Deserialize, Serialize};
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
    DID: did::Did = did::preset::Verifier,
    V: varsig::Header<C> = varsig::header::Preset,
    C: Codec + TryFrom<u64> + Into<u64> = varsig::encoding::Preset,
> {
    pub varsig_header: V,
    pub payload: Payload<A, DID>,
    pub signature: DID::Signature,
    _marker: std::marker::PhantomData<C>,
}

impl<
        A: Clone + ToCommand,
        DID: Clone + did::Did,
        V: Clone + varsig::Header<C>,
        C: Codec + TryFrom<u64> + Into<u64>,
    > Encode<C> for Invocation<A, DID, V, C>
where
    Ipld: Encode<C>,
    Named<Ipld>: From<A> + From<Payload<A, DID>>,
    Payload<A, DID>: TryFrom<Named<Ipld>>,
{
    fn encode<W: std::io::Write>(&self, c: C, w: &mut W) -> Result<(), libipld_core::error::Error> {
        self.to_ipld_envelope().encode(c, w)
    }
}

impl<A: Clone, DID: Did + Clone, V: varsig::Header<C>, C: Codec + TryFrom<u64> + Into<u64>>
    Invocation<A, DID, V, C>
where
    Ipld: Encode<C>,
{
    pub fn new(varsig_header: V, signature: DID::Signature, payload: Payload<A, DID>) -> Self {
        Invocation {
            varsig_header,
            payload,
            signature,
            _marker: std::marker::PhantomData,
        }
    }

    pub fn audience(&self) -> &Option<DID> {
        &self.payload.audience
    }

    pub fn issuer(&self) -> &DID {
        &self.payload.issuer
    }

    pub fn subject(&self) -> &DID {
        &self.payload.subject
    }

    pub fn ability(&self) -> &A {
        &self.payload.ability
    }

    pub fn map_ability<F, Z: Clone>(self, f: F) -> Invocation<Z, DID, V, C>
    where
        F: FnOnce(A) -> Z,
    {
        Invocation::new(
            self.varsig_header,
            self.signature,
            self.payload.map_ability(f),
        )
    }

    pub fn proofs(&self) -> &Vec<Cid> {
        &self.payload.proofs
    }

    pub fn issued_at(&self) -> &Option<Timestamp> {
        &self.payload.issued_at
    }

    pub fn expiration(&self) -> &Option<Timestamp> {
        &self.payload.expiration
    }

    pub fn check_time(&self, now: SystemTime) -> Result<(), Expired> {
        self.payload.check_time(now)
    }
}

impl<A, DID: Did, V: varsig::Header<C>, C: Codec + TryFrom<u64> + Into<u64>> did::Verifiable<DID>
    for Invocation<A, DID, V, C>
{
    fn verifier(&self) -> &DID {
        &self.verifier()
    }
}

impl<
        A: Clone + ToCommand,
        DID: Did + Clone,
        V: varsig::Header<C> + Clone,
        C: Codec + TryFrom<u64> + Into<u64>,
    > From<Invocation<A, DID, V, C>> for Ipld
where
    Named<Ipld>: From<A>,
    Payload<A, DID>: TryFrom<Named<Ipld>>,
{
    fn from(invocation: Invocation<A, DID, V, C>) -> Self {
        invocation.to_ipld_envelope()
    }
}

impl<
        A: Clone + ToCommand,
        DID: Did + Clone,
        V: varsig::Header<C> + Clone,
        C: Codec + TryFrom<u64> + Into<u64>,
    > Envelope for Invocation<A, DID, V, C>
where
    Named<Ipld>: From<A>,
    Payload<A, DID>: TryFrom<Named<Ipld>>,
{
    type DID = DID;
    type Payload = Payload<A, DID>;
    type VarsigHeader = V;
    type Encoder = C;

    fn construct(
        varsig_header: V,
        signature: DID::Signature,
        payload: Payload<A, DID>,
    ) -> Invocation<A, DID, V, C> {
        Invocation {
            varsig_header,
            payload,
            signature,
            _marker: std::marker::PhantomData,
        }
    }

    fn varsig_header(&self) -> &V {
        &self.varsig_header
    }

    fn payload(&self) -> &Payload<A, DID> {
        &self.payload
    }

    fn signature(&self) -> &DID::Signature {
        &self.signature
    }

    fn verifier(&self) -> &DID {
        &self.payload.issuer
    }
}

impl<
        A: Clone + ToCommand,
        DID: Did + Clone,
        V: varsig::Header<C> + Clone,
        C: Codec + TryFrom<u64> + Into<u64>,
    > Serialize for Invocation<A, DID, V, C>
where
    Named<Ipld>: From<A>,
    Payload<A, DID>: TryFrom<Named<Ipld>>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_ipld_envelope().serialize(serializer)
    }
}

impl<
        'de,
        A: Clone + ToCommand,
        DID: Did + Clone,
        V: varsig::Header<C> + Clone,
        C: Codec + TryFrom<u64> + Into<u64>,
    > Deserialize<'de> for Invocation<A, DID, V, C>
where
    Named<Ipld>: From<A>,
    Payload<A, DID>: TryFrom<Named<Ipld>>,
    <Payload<A, DID> as TryFrom<Named<Ipld>>>::Error: std::fmt::Display,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let ipld = Ipld::deserialize(deserializer)?;
        Self::try_from_ipld_envelope(ipld).map_err(serde::de::Error::custom)
    }
}
