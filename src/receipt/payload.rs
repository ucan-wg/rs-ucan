//! The payload (non-signature) portion of a response from an [`Invocation`].
//!
//! [`Invocation`]: crate::invocation::Invocation

use super::responds::Responds;
use crate::{
    ability::arguments,
    capsule::Capsule,
    crypto::Nonce,
    did::{Did, Verifiable},
    time::Timestamp,
};
use libipld_core::{cid::Cid, error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{
    de::{self, MapAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Serialize, Serializer,
};
use std::{collections::BTreeMap, fmt, fmt::Debug};

#[cfg(feature = "test_utils")]
use proptest::prelude::*;

#[cfg(feature = "test_utils")]
use crate::ipld;

#[cfg(feature = "test_utils")]
use crate::ipld::cid;

impl<T: Responds, DID: Did> Verifiable<DID> for Payload<T, DID> {
    fn verifier(&self) -> &DID {
        &self.issuer
    }
}

/// The payload (non-signature) portion of a response from an [`Invocation`].
///
/// [`Invocation`]: crate::invocation::Invocation
#[derive(Debug, Clone, PartialEq)]
pub struct Payload<T: Responds, DID: Did> {
    /// The issuer of the [`Receipt`]. This [`Did`] *must* match the signature on
    /// the outer layer of [`Receipt`].
    ///
    /// [`Receipt`]: super::Receipt
    pub issuer: DID,

    /// The [`Cid`] of the [`Invocation`] that was run.
    ///
    /// [`Invocation`]: crate::invocation::Invocation
    pub ran: Cid,

    /// The output of the [`Invocation`]. This is always of
    /// the form `{"ok": ...}` or `{"err": ...}`.
    ///
    /// [`Invocation`]: crate::invocation::Invocation
    pub out: Result<T::Success, arguments::Named<Ipld>>,

    /// Any further [`Invocation`]s that the `ran` [`Invocation`]
    /// requested to be queued next.
    ///
    /// [`Invocation`]: crate::invocation::Invocation
    pub next: Vec<Cid>,

    /// An optional proof chain authorizing a different [`Did`] to
    /// be the receipt `iss` than the audience (or subject) of the
    /// [`Invocation`] that was run.
    ///
    /// [`Invocation`]: crate::invocation::Invocation
    pub proofs: Vec<Cid>,

    /// Extensible, free-form fields.
    pub metadata: BTreeMap<String, Ipld>,

    /// A [cryptographic nonce] to ensure that the UCAN's [`Cid`] is unique.
    ///
    /// [cryptographic nonce]: https://en.wikipedia.org/wiki/Cryptographic_nonce
    /// [`Cid`]: libipld_core::cid::Cid
    pub nonce: Nonce,

    /// An optional [Unix timestamp] (wall-clock time) at which the
    /// receipt claims to have been issued at.
    ///
    /// [Unix timestamp]: https://en.wikipedia.org/wiki/Unix_time
    pub issued_at: Option<Timestamp>,
}

impl<T: Responds, DID: Did> Capsule for Payload<T, DID> {
    const TAG: &'static str = "ucan/r@1.0.0-rc.1";
}

impl<T: Responds, DID: Did + Clone> Serialize for Payload<T, DID>
where
    T::Success: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let field_count = 7 + self.issued_at.is_some() as usize;

        let mut state = serializer.serialize_struct("receipt::Payload", field_count)?;

        state.serialize_field("iss", &self.issuer.clone().into().as_str())?;
        state.serialize_field("ran", &self.ran)?;
        state.serialize_field("out", &self.out)?;
        state.serialize_field("next", &self.next)?;
        state.serialize_field("prf", &self.proofs)?;
        state.serialize_field("meta", &self.metadata)?;
        state.serialize_field("nonce", &self.nonce)?;
        state.serialize_field("iat", &self.issued_at)?;

        state.end()
    }
}

impl<'de, T: Responds, DID: Did + Deserialize<'de>> Deserialize<'de> for Payload<T, DID>
where
    T::Success: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ReceiptPayloadVisitor<T, DID>(std::marker::PhantomData<(T, DID)>);

        const FIELDS: &'static [&'static str] =
            &["iss", "ran", "out", "next", "prf", "meta", "nonce", "iat"];

        impl<'de, T: Responds, DID: Did + Deserialize<'de>> Visitor<'de> for ReceiptPayloadVisitor<T, DID>
        where
            T::Success: Deserialize<'de>,
        {
            type Value = Payload<T, DID>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("struct delegation::Payload")
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'de>,
            {
                let mut issuer = None;
                let mut ran = None;
                let mut out = None;
                let mut next = None;
                let mut proofs = None;
                let mut metadata = None;
                let mut nonce = None;
                let mut issued_at = None;

                while let Some(key) = map.next_key()? {
                    match key {
                        "iss" => {
                            if issuer.is_some() {
                                return Err(de::Error::duplicate_field("iss"));
                            }
                            issuer = Some(map.next_value()?);
                        }
                        "ran" => {
                            if ran.is_some() {
                                return Err(de::Error::duplicate_field("ran"));
                            }
                            ran = Some(map.next_value()?);
                        }
                        "out" => {
                            if out.is_some() {
                                return Err(de::Error::duplicate_field("out"));
                            }
                            out = Some(map.next_value()?);
                        }
                        "next" => {
                            if next.is_some() {
                                return Err(de::Error::duplicate_field("next"));
                            }
                            next = Some(map.next_value()?);
                        }
                        "prf" => {
                            if proofs.is_some() {
                                return Err(de::Error::duplicate_field("prf"));
                            }
                            proofs = Some(map.next_value()?);
                        }
                        "meta" => {
                            if metadata.is_some() {
                                return Err(de::Error::duplicate_field("meta"));
                            }
                            metadata = Some(map.next_value()?);
                        }
                        "nonce" => {
                            if nonce.is_some() {
                                return Err(de::Error::duplicate_field("nonce"));
                            }
                            nonce = Some(map.next_value()?);
                        }
                        "iat" => {
                            if issued_at.is_some() {
                                return Err(de::Error::duplicate_field("iat"));
                            }
                            issued_at = map.next_value()?;
                        }
                        other => {
                            return Err(de::Error::unknown_field(other, FIELDS));
                        }
                    }
                }

                Ok(Payload {
                    issuer: issuer.ok_or(de::Error::missing_field("iss"))?,
                    ran: ran.ok_or(de::Error::missing_field("ran"))?,
                    out: out.ok_or(de::Error::missing_field("out"))?,
                    next: next.ok_or(de::Error::missing_field("next"))?,
                    proofs: proofs.ok_or(de::Error::missing_field("prf"))?,
                    metadata: metadata.ok_or(de::Error::missing_field("meta"))?,
                    nonce: nonce.ok_or(de::Error::missing_field("nonce"))?,
                    issued_at,
                })
            }
        }

        deserializer.deserialize_struct(
            "ReceiptPayload",
            FIELDS,
            ReceiptPayloadVisitor(Default::default()),
        )
    }
}

impl<T: Responds, DID: Did> From<Payload<T, DID>> for Ipld {
    fn from(payload: Payload<T, DID>) -> Self {
        payload.into()
    }
}

impl<T: Responds, DID: Did> TryFrom<Ipld> for Payload<T, DID>
where
    Payload<T, DID>: for<'de> Deserialize<'de>,
{
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

#[cfg(feature = "test_utils")]
impl<T: Responds + Debug, DID: Arbitrary + Did> Arbitrary for Payload<T, DID>
where
    T::Success: Arbitrary + 'static,
    DID::Parameters: Clone,
    DID::Strategy: 'static,
{
    type Parameters = (<T::Success as Arbitrary>::Parameters, DID::Parameters);
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with((t_params, did_params): Self::Parameters) -> Self::Strategy {
        (
            DID::arbitrary_with(did_params),
            cid::Newtype::arbitrary(),
            prop_oneof![
                T::Success::arbitrary_with(t_params).prop_map(Result::Ok),
                arguments::Named::arbitrary().prop_map(Result::Err),
            ],
            prop::collection::vec(cid::Newtype::arbitrary(), 0..25),
            prop::collection::vec(cid::Newtype::arbitrary(), 0..25),
            prop::collection::btree_map(".*", ipld::Newtype::arbitrary(), 0..50),
            Nonce::arbitrary(),
            prop::option::of(Timestamp::arbitrary()),
        )
            .prop_map(
                |(issuer, ran, out, next, proofs, newtype_metadata, nonce, issued_at)| Payload {
                    issuer,
                    ran: ran.cid,
                    out,
                    next: next.into_iter().map(|nt| nt.cid).collect(),
                    proofs: proofs.into_iter().map(|nt| nt.cid).collect(),
                    metadata: newtype_metadata
                        .into_iter()
                        .map(|(k, v)| (k, v.0))
                        .collect(),
                    nonce,
                    issued_at,
                },
            )
            .boxed()
    }
}
