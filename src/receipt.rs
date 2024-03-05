//! A [`Receipt`] is the (optional) response from an [`Invocation`][`crate::invocation::Invocation`].
//!
//! - [`Receipt`]s are the result of an [`Invocation`][`crate::invocation::Invocation`].
//! - [`Payload`] contains the pimary semantic information for a [`Receipt`].
//! - [`Store`] is the storage interface for [`Receipt`]s.
//! - [`Responds`] associates the response success type to an [Ability][crate::ability].

mod payload;
mod responds;

pub mod store;

pub use payload::Payload;
pub use responds::Responds;
pub use store::Store;

use crate::{
    ability,
    crypto::{signature, varsig},
    did::{self, Did},
};
use libipld_core::{
    codec::{Codec, Encode},
    ipld::Ipld,
};
use serde::{Deserialize, Serialize};

/// The complete, signed receipt of an [`Invocation`][`crate::invocation::Invocation`].
#[derive(Clone, Debug, PartialEq)]
pub struct Receipt<T: Responds, DID: Did, V: varsig::Header<C>, C: Codec + Into<u32> + TryFrom<u32>>(
    pub signature::Envelope<Payload<T, DID>, DID, V, C>,
);

/// An alias for the [`Receipt`] type with the library preset
/// [`Did`](crate::did)s and [Abilities](crate::ability).
pub type Preset = Receipt<
    ability::preset::Preset,
    did::preset::Verifier,
    varsig::header::Preset,
    varsig::encoding::Preset,
>;

impl<T: Responds, DID: did::Did, V: varsig::Header<Enc>, Enc: Codec + Into<u32> + TryFrom<u32>>
    Receipt<T, DID, V, Enc>
{
    /// Returns the [`Payload`] of the [`Receipt`].
    pub fn payload(&self) -> &Payload<T, DID> {
        &self.0.payload
    }

    /// Returns the [`signature::Envelope`] of the [`Receipt`].
    pub fn signature(&self) -> &DID::Signature {
        &self.0.signature
    }

    pub fn varsig_encode(self, w: &mut Vec<u8>) -> Result<(), libipld_core::error::Error>
    where
        Ipld: Encode<Enc>,
    {
        self.0.varsig_encode(w)
    }
}

impl<T: Responds, DID: Did, V: varsig::Header<Enc>, Enc: Codec + TryFrom<u32> + Into<u32>>
    did::Verifiable<DID> for Receipt<T, DID, V, Enc>
{
    fn verifier(&self) -> &DID {
        &self.0.verifier()
    }
}

impl<T: Responds, DID: Did, V: varsig::Header<Enc>, Enc: Codec + TryFrom<u32> + Into<u32>>
    From<Receipt<T, DID, V, Enc>> for Ipld
{
    fn from(invocation: Receipt<T, DID, V, Enc>) -> Self {
        invocation.0.into()
    }
}

impl<T: Responds, DID: Did, V: varsig::Header<Enc>, Enc: Codec + TryFrom<u32> + Into<u32>>
    TryFrom<Ipld> for Receipt<T, DID, V, Enc>
where
    Payload<T, DID>: TryFrom<Ipld>,
{
    type Error = <signature::Envelope<Payload<T, DID>, DID, V, Enc> as TryFrom<Ipld>>::Error;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        signature::Envelope::try_from(ipld).map(Receipt)
    }
}

impl<T: Responds, DID: Did, V: varsig::Header<Enc>, Enc: Codec + TryFrom<u32> + Into<u32>> Serialize
    for Receipt<T, DID, V, Enc>
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, T: Responds, DID: Did, V: varsig::Header<Enc>, Enc: Codec + TryFrom<u32> + Into<u32>>
    Deserialize<'de> for Receipt<T, DID, V, Enc>
where
    Payload<T, DID>: TryFrom<Ipld>,
    <Payload<T, DID> as TryFrom<Ipld>>::Error: std::fmt::Display,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        signature::Envelope::deserialize(deserializer).map(Receipt)
    }
}
