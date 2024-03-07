//! A [`Receipt`] is the (optional) response from an [`Invocation`][`crate::invocation::Invocation`].
//!
//! - [`Receipt`]s are the result of an [`Invocation`][`crate::invocation::Invocation`].
//! - [`Payload`] contains the pimary semantic information for a [`Receipt`].
//! - [`Store`] is the storage interface for [`Receipt`]s.
//! - [`Responds`] associates the response success type to an [Ability][crate::ability].

mod payload;
mod responds;

pub mod store;

pub use payload::*;
pub use responds::Responds;
pub use store::Store;

use crate::{
    crypto::{signature::Envelope, varsig},
    did::{self, Did},
};
use libipld_core::{codec::Codec, ipld::Ipld};
use serde::{Deserialize, Serialize};

/// The complete, signed receipt of an [`Invocation`][`crate::invocation::Invocation`].
#[derive(Clone, Debug, PartialEq)]
pub struct Receipt<
    T: Responds,
    DID: Did = did::preset::Verifier,
    V: varsig::Header<C> = varsig::header::Preset,
    C: Codec + Into<u64> + TryFrom<u64> = varsig::encoding::Preset,
> {
    pub varsig_header: V,
    pub signature: DID::Signature,
    pub payload: Payload<T, DID>,

    _marker: std::marker::PhantomData<C>,
}

impl<T: Responds, DID: Did, V: varsig::Header<C>, C: Codec + TryFrom<u64> + Into<u64>>
    did::Verifiable<DID> for Receipt<T, DID, V, C>
{
    fn verifier(&self) -> &DID {
        &self.verifier()
    }
}

impl<
        T: Responds + Clone,
        DID: Did + Clone,
        V: varsig::Header<C> + Clone,
        C: Codec + TryFrom<u64> + Into<u64>,
    > From<Receipt<T, DID, V, C>> for Ipld
where
    Payload<T, DID>: TryFrom<Ipld>,
{
    fn from(rec: Receipt<T, DID, V, C>) -> Self {
        rec.to_ipld_envelope()
    }
}

impl<
        T: Responds + Clone,
        DID: Did + Clone,
        V: varsig::Header<C> + Clone,
        C: Codec + TryFrom<u64> + Into<u64>,
    > Envelope for Receipt<T, DID, V, C>
where
    Payload<T, DID>: TryFrom<Ipld>,
{
    type DID = DID;
    type Payload = Payload<T, DID>;
    type VarsigHeader = V;
    type Encoder = C;

    fn construct(
        varsig_header: V,
        signature: DID::Signature,
        payload: Payload<T, DID>,
    ) -> Receipt<T, DID, V, C> {
        Receipt {
            varsig_header,
            payload,
            signature,
            _marker: std::marker::PhantomData,
        }
    }

    fn varsig_header(&self) -> &V {
        &self.varsig_header
    }

    fn payload(&self) -> &Payload<T, DID> {
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
        T: Responds + Clone,
        DID: Did + Clone,
        V: varsig::Header<C> + Clone,
        C: Codec + TryFrom<u64> + Into<u64>,
    > Serialize for Receipt<T, DID, V, C>
where
    Payload<T, DID>: TryFrom<Ipld>,
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
        T: Responds + Clone,
        DID: Did + Clone,
        V: varsig::Header<C> + Clone,
        C: Codec + TryFrom<u64> + Into<u64>,
    > Deserialize<'de> for Receipt<T, DID, V, C>
where
    Payload<T, DID>: TryFrom<Ipld>,
    <Payload<T, DID> as TryFrom<Ipld>>::Error: std::fmt::Display,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let ipld = Ipld::deserialize(deserializer)?;
        Self::try_from_ipld_envelope(ipld).map_err(serde::de::Error::custom)
    }
}
