//! The payload (non-signature) portion of a response from an [`Invocation`].
//!
//! [`Invocation`]: crate::invocation::Invocation

use super::responds::Responds;
use crate::{ability::arguments, capsule::Capsule, did::Did, nonce::Nonce, time::Timestamp};
use libipld_core::{cid::Cid, error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{
    de::{self, DeserializeOwned, MapAccess, Visitor},
    ser::SerializeStruct,
    Deserialize, Serialize, Serializer,
};
use std::{collections::BTreeMap, fmt, fmt::Debug};

/// The payload (non-signature) portion of a response from an [`Invocation`].
///
/// [`Invocation`]: crate::invocation::Invocation
#[derive(Debug, Clone, PartialEq)]
pub struct Payload<T: Responds> {
    /// The issuer of the [`Receipt`].
    ///
    /// This [`Did`] *must* match the signature on
    /// the outer layer of [`Receipt`].
    ///
    /// [`Receipt`]: super::Receipt
    pub issuer: Did,

    /// The [`Cid`] of the [`Invocation`] that was run.
    ///
    /// [`Invocation`]: crate::invocation::Invocation
    pub ran: Cid,

    /// The output of the [`Invocation`].
    ///
    /// This is always of the form `{"ok": ...}` or `{"err": ...}`.
    ///
    /// [`Invocation`]: crate::invocation::Invocation
    pub out: Result<T::Success, arguments::Named<Ipld>>,

    /// Any further [`Invocation`]s that the `ran` [`Invocation`]
    /// requested to be queued next.
    ///
    /// [`Invocation`]: crate::invocation::Invocation
    pub next: Vec<Cid>, // FIXME rename here or in spec?

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

impl<T: Responds> Capsule for Payload<T> {
    const TAG: &'static str = "ucan/r/1.0.0-rc.1";
}

impl<T: Responds> Serialize for Payload<T>
where
    T::Success: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut state = serializer.serialize_struct("receipt::Payload", 8)?;
        state.serialize_field("iss", &self.issuer)?;
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

impl<'de, T: Responds> Deserialize<'de> for Payload<T>
where
    T::Success: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct ReceiptPayloadVisitor<T>(std::marker::PhantomData<T>);

        const FIELDS: &'static [&'static str] =
            &["iss", "ran", "out", "next", "prf", "meta", "nonce", "iat"];

        impl<'de, T: Responds> Visitor<'de> for ReceiptPayloadVisitor<T>
        where
            T::Success: Deserialize<'de>,
        {
            type Value = Payload<T>;

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
                    issuer: issuer.ok_or_else(|| de::Error::missing_field("iss"))?,
                    ran: ran.ok_or_else(|| de::Error::missing_field("ran"))?,
                    out: out.ok_or_else(|| de::Error::missing_field("out"))?,
                    next: next.ok_or_else(|| de::Error::missing_field("next"))?,
                    proofs: proofs.ok_or_else(|| de::Error::missing_field("prf"))?,
                    metadata: metadata.ok_or_else(|| de::Error::missing_field("meta"))?,
                    nonce: nonce.ok_or_else(|| de::Error::missing_field("nonce"))?,
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

impl<T: Responds> From<Payload<T>> for Ipld {
    fn from(payload: Payload<T>) -> Self {
        payload.into()
    }
}

impl<T: Responds> TryFrom<Ipld> for Payload<T>
where
    T::Success: DeserializeOwned,
{
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}
