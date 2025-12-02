//! UCAN Delegation
//!
//! The spec for UCAN Delegations can be found at
//! [the GitHub repo](https://github.com/ucan-wg/invocation/).

pub mod builder;
pub mod policy;
pub mod store;
pub mod subject;

use self::subject::DelegatedSubject;
use crate::{
    cid::to_dagcbor_cid,
    command::Command,
    crypto::nonce::Nonce,
    did::{Did, DidSigner},
    envelope::{payload_tag::PayloadTag, Envelope},
    time::timestamp::Timestamp,
    unset::Unset,
};
use builder::DelegationBuilder;
use ipld_core::{cid::Cid, ipld::Ipld};
use policy::predicate::Predicate;
use serde::{
    de::{self, MapAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};
use std::{borrow::Cow, collections::BTreeMap, fmt::Debug};
use varsig::verify::Verify;

/// Top-level UCAN Delegation.
#[derive(Clone)]
pub struct Delegation<D: Did>(
    Envelope<D::VarsigConfig, DelegationPayload<D>, <D::VarsigConfig as Verify>::Signature>,
);

impl<D: Did> Delegation<D> {
    /// Creates a blank [`DelegationBuilder`] instance.
    #[must_use]
    pub const fn builder<S: DidSigner<Did = D>>() -> DelegationBuilder<S, Unset, Unset, Unset, Unset>
    {
        DelegationBuilder::new()
    }

    /// Getter for the `issuer` field.
    pub const fn issuer(&self) -> &D {
        &self.0 .1.payload.issuer
    }

    /// Getter for the `audience` field.
    pub const fn audience(&self) -> &D {
        &self.0 .1.payload.audience
    }

    /// Getter for the `subject` field.
    pub const fn subject(&self) -> &DelegatedSubject<D> {
        &self.0 .1.payload.subject
    }

    /// Getter for the `command` field.
    pub const fn command(&self) -> &Command {
        &self.0 .1.payload.command
    }

    /// Getter for the `policy` field.
    pub const fn policy(&self) -> &Vec<Predicate> {
        &self.0 .1.payload.policy
    }

    /// Getter for the `expiration` field.
    pub const fn expiration(&self) -> Option<Timestamp> {
        self.0 .1.payload.expiration
    }

    /// Getter for the `not_before` field.
    pub const fn not_before(&self) -> Option<Timestamp> {
        self.0 .1.payload.not_before
    }

    /// Getter for the `meta` field.
    pub const fn meta(&self) -> &BTreeMap<String, Ipld> {
        &self.0 .1.payload.meta
    }

    /// Getter for the `nonce` field.
    pub const fn nonce(&self) -> &Nonce {
        &self.0 .1.payload.nonce
    }

    /// Compute the CID for this delegation.
    pub fn to_cid(&self) -> Cid {
        to_dagcbor_cid(&self)
    }
}

impl<D: Did> Debug for Delegation<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Delegation").field(&self.0).finish()
    }
}

impl<D: Did> Serialize for Delegation<D> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, I: Did> Deserialize<'de> for Delegation<I>
where
    <I::VarsigConfig as Verify>::Signature: for<'ze> Deserialize<'ze>,
{
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let envelope = Envelope::<_, _, _>::deserialize(deserializer)?;
        Ok(Delegation(envelope))
    }
}

/// UCAN Delegation
///
/// Grant or delegate a UCAN capability to another. This type implements the
/// [UCAN Delegation spec](https://github.com/ucan-wg/delegation/README.md).
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct DelegationPayload<D: Did> {
    #[serde(rename = "iss")]
    pub(crate) issuer: D,

    #[serde(rename = "aud")]
    pub(crate) audience: D,

    #[serde(rename = "sub")]
    pub(crate) subject: DelegatedSubject<D>,

    #[serde(rename = "cmd")]
    pub(crate) command: Command,

    #[serde(rename = "pol")]
    pub(crate) policy: Vec<Predicate>,

    #[serde(rename = "exp")]
    pub(crate) expiration: Option<Timestamp>,

    #[serde(rename = "nbf")]
    pub(crate) not_before: Option<Timestamp>,

    pub(crate) meta: BTreeMap<String, Ipld>,
    pub(crate) nonce: Nonce,
}

impl<D: Did> DelegationPayload<D> {
    /// Getter for the `issuer` field.
    pub const fn issuer(&self) -> &D {
        &self.issuer
    }

    /// Getter for the `audience` field.
    pub const fn audience(&self) -> &D {
        &self.audience
    }

    /// Getter for the `subject` field.
    pub const fn subject(&self) -> &DelegatedSubject<D> {
        &self.subject
    }

    /// Getter for the `command` field.
    pub const fn command(&self) -> &Command {
        &self.command
    }

    /// Getter for the `policy` field.
    pub const fn policy(&self) -> &Vec<Predicate> {
        &self.policy
    }

    /// Getter for the `expiration` field.
    pub const fn expiration(&self) -> Option<Timestamp> {
        self.expiration
    }

    /// Getter for the `not_before` field.
    pub const fn not_before(&self) -> Option<Timestamp> {
        self.not_before
    }

    /// Getter for the `meta` field.
    pub const fn meta(&self) -> &BTreeMap<String, Ipld> {
        &self.meta
    }

    /// Getter for the `nonce` field.
    pub const fn nonce(&self) -> &Nonce {
        &self.nonce
    }
}

impl<'de, D> Deserialize<'de> for DelegationPayload<D>
where
    D: Did,
    DelegatedSubject<D>: Deserialize<'de>,
    Predicate: Deserialize<'de>,
    Timestamp: Deserialize<'de>,
    Nonce: Deserialize<'de>,
    Ipld: Deserialize<'de>,
{
    #[allow(clippy::too_many_lines)]
    fn deserialize<T>(deserializer: T) -> Result<Self, T::Error>
    where
        T: Deserializer<'de>,
    {
        struct PayloadVisitor<D: Did>(std::marker::PhantomData<D>);

        impl<'de, D> Visitor<'de> for PayloadVisitor<D>
        where
            D: Did,
            DelegatedSubject<D>: Deserialize<'de>,
            Predicate: Deserialize<'de>,
            Timestamp: Deserialize<'de>,
            Nonce: Deserialize<'de>,
            Ipld: Deserialize<'de>,
        {
            type Value = DelegationPayload<D>;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a map with keys iss,aud,sub,cmd,pol,exp,nbf,meta,nonce")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut issuer: Option<D> = None;
                let mut audience: Option<D> = None;
                let mut subject: Option<DelegatedSubject<D>> = None;
                let mut command: Option<Command> = None;
                let mut policy: Option<Vec<Predicate>> = None;
                let mut expiration: Option<Option<Timestamp>> = None;
                let mut not_before: Option<Option<Timestamp>> = None;
                let mut meta: Option<BTreeMap<String, Ipld>> = None;
                let mut nonce: Option<Nonce> = None;

                while let Some(key) = map.next_key::<Cow<'de, str>>()? {
                    match key.as_ref() {
                        "iss" => {
                            if issuer.is_some() {
                                return Err(de::Error::duplicate_field("iss"));
                            }
                            issuer = Some(map.next_value()?);
                        }
                        "aud" => {
                            if audience.is_some() {
                                return Err(de::Error::duplicate_field("aud"));
                            }
                            audience = Some(map.next_value()?);
                        }
                        "sub" => {
                            if subject.is_some() {
                                return Err(de::Error::duplicate_field("sub"));
                            }
                            subject = Some(map.next_value()?);
                        }
                        "cmd" => {
                            if command.is_some() {
                                return Err(de::Error::duplicate_field("cmd"));
                            }
                            let cmd: Command = map.next_value()?;
                            command = Some(cmd);
                        }
                        "pol" => {
                            if policy.is_some() {
                                return Err(de::Error::duplicate_field("pol"));
                            }
                            policy = Some(map.next_value()?);
                        }
                        "exp" => {
                            if expiration.is_some() {
                                return Err(de::Error::duplicate_field("exp"));
                            }
                            expiration = Some(map.next_value()?);
                        }
                        "nbf" => {
                            if not_before.is_some() {
                                return Err(de::Error::duplicate_field("nbf"));
                            }
                            not_before = Some(map.next_value()?);
                        }
                        "meta" => {
                            if meta.is_some() {
                                return Err(de::Error::duplicate_field("meta"));
                            }
                            meta = Some(map.next_value()?);
                        }
                        "nonce" => {
                            if nonce.is_some() {
                                return Err(de::Error::duplicate_field("nonce"));
                            }
                            let ipld: Ipld = map.next_value()?;
                            let v = match ipld {
                                Ipld::Bytes(b) => b,
                                Ipld::String(s) => {
                                    return Err(de::Error::invalid_type(
                                        de::Unexpected::Str(&s),
                                        &"bytes",
                                    ));
                                }
                                Ipld::Integer(i) => {
                                    return Err(de::Error::invalid_type(
                                        de::Unexpected::Other(&i.to_string()),
                                        &"bytes",
                                    ));
                                }
                                Ipld::Float(f) => {
                                    return Err(de::Error::invalid_type(
                                        de::Unexpected::Float(f),
                                        &"bytes",
                                    ));
                                }
                                Ipld::Bool(b) => {
                                    return Err(de::Error::invalid_type(
                                        de::Unexpected::Bool(b),
                                        &"bytes",
                                    ));
                                }
                                Ipld::Null => {
                                    return Err(de::Error::invalid_type(
                                        de::Unexpected::Unit,
                                        &"bytes",
                                    ));
                                }
                                Ipld::List(_) => {
                                    return Err(de::Error::invalid_type(
                                        de::Unexpected::Other("list"),
                                        &"bytes",
                                    ));
                                }
                                Ipld::Map(_) => {
                                    return Err(de::Error::invalid_type(
                                        de::Unexpected::Map,
                                        &"bytes",
                                    ));
                                }
                                Ipld::Link(_) => {
                                    return Err(de::Error::invalid_type(
                                        de::Unexpected::Other("link"),
                                        &"bytes",
                                    ));
                                }
                            };

                            if let Ok(arr) = <[u8; 16]>::try_from(v.clone()) {
                                nonce = Some(Nonce::Nonce16(arr));
                            } else {
                                nonce = Some(Nonce::Custom(v));
                            }
                        }
                        other => {
                            return Err(de::Error::unknown_field(
                                other,
                                &[
                                    "iss", "aud", "sub", "cmd", "pol", "exp", "nbf", "meta",
                                    "nonce",
                                ],
                            ));
                        }
                    }
                }

                let issuer = issuer.ok_or_else(|| de::Error::missing_field("iss"))?;
                let audience = audience.ok_or_else(|| de::Error::missing_field("aud"))?;
                let subject = subject.ok_or_else(|| de::Error::missing_field("sub"))?;
                let command = command.ok_or_else(|| de::Error::missing_field("cmd"))?;
                let policy = policy.ok_or_else(|| de::Error::missing_field("pol"))?;
                let nonce = nonce.ok_or_else(|| de::Error::missing_field("nonce"))?;

                Ok(DelegationPayload {
                    issuer,
                    audience,
                    subject,
                    command,
                    policy,
                    nonce,
                    expiration: expiration.unwrap_or(None),
                    not_before: not_before.unwrap_or(None),
                    meta: meta.unwrap_or_default(),
                })
            }
        }

        deserializer.deserialize_map(PayloadVisitor::<D>(std::marker::PhantomData))
    }
}

impl<D: Did> PayloadTag for DelegationPayload<D> {
    fn spec_id() -> &'static str {
        "dlg"
    }

    fn version() -> &'static str {
        "1.0.0-rc.1"
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::did::{Ed25519Did, Ed25519Signer};

    use base64::prelude::*;
    use testresult::TestResult;

    #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
    struct EdKey(ed25519_dalek::VerifyingKey);

    #[test]
    fn issuer_round_trip() -> TestResult {
        let iss: Ed25519Signer = ed25519_dalek::SigningKey::from_bytes(&[0u8; 32]).into();
        let aud: Ed25519Did = ed25519_dalek::VerifyingKey::from_bytes(&[0u8; 32])
            .unwrap()
            .into();
        let sub: Ed25519Did = ed25519_dalek::VerifyingKey::from_bytes(&[0u8; 32])
            .unwrap()
            .into();

        let builder: DelegationBuilder<
            Ed25519Signer,
            Ed25519Signer,
            Ed25519Did,
            DelegatedSubject<Ed25519Did>,
            Command,
        > = DelegationBuilder::new()
            .issuer(iss.clone())
            .audience(aud)
            .subject(DelegatedSubject::Specific(sub))
            .command(vec!["read".to_string(), "write".to_string()]);

        let delegation = builder.try_build()?;

        assert_eq!(delegation.issuer().to_string(), iss.to_string());
        Ok(())
    }

    #[test]
    fn delegation_b64_fixture_roundtrip() -> TestResult {
        // Sample delegation with sub: null, cmd: "/", exp: null, meta: {}
        let b64 = "glhA0rict5hwniXnh54Y7b0v/ZEDNSlPdBx0rsoWDYC2Ylv+UzDr00s7ojPsfvNwrofqKItK911ZGJggZSkeQIB3DqJhaEg0Ae0B7QETcXN1Y2FuL2RsZ0AxLjAuMC1yYy4xqWNhdWR4OGRpZDprZXk6ejZNa2ZGSkJ4U0JGZ29BcVRRTFM3YlRmUDhNZ3lEeXB2YTVpNkNMNVBKTjhSSlpyY2NtZGEvY2V4cPZjaXNzeDhkaWQ6a2V5Ono2TWtyQXNxMU03dEVmUHZXNWRSMlVGQ3daU3pSTU5YWWVUVzh0R1pTS3ZVbTlFWmNuYmYaaSTxp2Nwb2yAY3N1YvZkbWV0YaBlbm9uY2VMVkDFeab+58p8SMpW";
        let bytes = BASE64_STANDARD.decode(b64)?;

        // Parse as Delegation
        let delegation: Delegation<Ed25519Did> = serde_ipld_dagcbor::from_slice(&bytes)?;

        // Verify fields parsed correctly
        assert_eq!(delegation.subject(), &DelegatedSubject::Any); // sub: null
        assert_eq!(delegation.command(), &vec![].into()); // cmd: "/"
        assert_eq!(delegation.expiration(), None); // exp: null
        assert!(delegation.not_before().is_some()); // nbf: 1764028839

        // Serialize back
        let reserialized = serde_ipld_dagcbor::to_vec(&delegation)?;

        // Verify byte-exact roundtrip
        assert_eq!(
            bytes, reserialized,
            "Reserialized bytes should match original"
        );

        // Deserialize again to verify roundtrip preserves all fields
        let roundtripped: Delegation<Ed25519Did> = serde_ipld_dagcbor::from_slice(&reserialized)?;
        assert_eq!(roundtripped.subject(), delegation.subject());
        assert_eq!(roundtripped.command(), delegation.command());
        assert_eq!(roundtripped.expiration(), delegation.expiration());
        assert_eq!(roundtripped.not_before(), delegation.not_before());
        assert_eq!(roundtripped.issuer(), delegation.issuer());
        assert_eq!(roundtripped.audience(), delegation.audience());

        Ok(())
    }

    #[test]
    fn delegation_payload_any_subject_serializes_to_null() -> TestResult {
        use crate::crypto::nonce::Nonce;

        let iss: Ed25519Did = ed25519_dalek::VerifyingKey::from_bytes(&[0u8; 32])
            .unwrap()
            .into();
        let aud: Ed25519Did = ed25519_dalek::VerifyingKey::from_bytes(&[0u8; 32])
            .unwrap()
            .into();

        let payload = DelegationPayload {
            issuer: iss,
            audience: aud,
            subject: DelegatedSubject::Any,
            command: Command::new(vec!["/".to_string()]),
            policy: vec![],
            expiration: None,
            not_before: None,
            meta: std::collections::BTreeMap::new(),
            nonce: Nonce::generate_16().unwrap(),
        };

        assert_eq!(payload.subject(), &DelegatedSubject::Any);

        // Serialize to CBOR
        let bytes = serde_ipld_dagcbor::to_vec(&payload)?;

        // Parse as IPLD to verify structure
        let ipld: ipld_core::ipld::Ipld = serde_ipld_dagcbor::from_slice(&bytes)?;

        // Verify sub is null in the serialized form
        if let ipld_core::ipld::Ipld::Map(map) = &ipld {
            let sub = map.get("sub").expect("sub field should exist");
            assert_eq!(
                sub,
                &ipld_core::ipld::Ipld::Null,
                "sub should be null for Any"
            );
        } else {
            panic!("Expected a map");
        }

        Ok(())
    }
}
