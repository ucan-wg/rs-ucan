//! UCAN Delegation
//!
//! The spec for UCAN Delegations can be found at
//! [the GitHub repo](https://github.com/ucan-wg/invocation/).

pub mod builder;
pub mod policy;
pub mod subject;

use self::subject::DelegatedSubject;
use crate::{
    crypto::nonce::Nonce,
    did::{Did, DidSigner},
    envelope::Envelope,
    time::timestamp::Timestamp,
    unset::Unset,
};
use builder::DelegationBuilder;
use ipld_core::ipld::Ipld;
use policy::predicate::Predicate;
use serde::{
    de::{MapAccess, Visitor},
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
    pub const fn command(&self) -> &Vec<String> {
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
    pub(crate) command: Vec<String>,

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
    pub const fn command(&self) -> &Vec<String> {
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
                let mut command: Option<Vec<String>> = None;
                let mut policy: Option<Vec<Predicate>> = None;
                let mut expiration: Option<Option<Timestamp>> = None;
                let mut not_before: Option<Option<Timestamp>> = None;
                let mut meta: BTreeMap<String, Ipld> = BTreeMap::new();
                let mut nonce: Option<Nonce> = None;

                while let Some(key) = map.next_key::<Cow<'de, str>>()? {
                    match key.as_ref() {
                        "iss" => {
                            if issuer.is_some() {
                                return Err(serde::de::Error::duplicate_field("iss"));
                            }
                            issuer = Some(map.next_value()?);
                        }
                        "aud" => {
                            if audience.is_some() {
                                return Err(serde::de::Error::duplicate_field("aud"));
                            }
                            audience = Some(map.next_value()?);
                        }
                        "sub" => {
                            if subject.is_some() {
                                return Err(serde::de::Error::duplicate_field("sub"));
                            }
                            subject = Some(map.next_value()?);
                        }
                        "cmd" => {
                            if command.is_some() {
                                return Err(serde::de::Error::duplicate_field("cmd"));
                            }
                            let s: String = map.next_value()?;
                            command = Some(s.split("/").map(ToString::to_string).collect());
                        }
                        "pol" => {
                            if policy.is_some() {
                                return Err(serde::de::Error::duplicate_field("pol"));
                            }
                            policy = Some(map.next_value()?);
                        }
                        "exp" => {
                            if expiration.is_some() {
                                return Err(serde::de::Error::duplicate_field("exp"));
                            }
                            expiration = Some(map.next_value()?);
                        }
                        "nbf" => {
                            if not_before.is_some() {
                                return Err(serde::de::Error::duplicate_field("nbf"));
                            }
                            not_before = Some(map.next_value()?);
                        }
                        "meta" => {
                            // If the payload already has a meta map, we *merge* into it.
                            let incoming: BTreeMap<String, Ipld> = map.next_value()?;
                            for (k, v) in incoming {
                                // last one wins if duplicated inside meta
                                meta.insert(k, v);
                            }
                        }
                        "nonce" => {
                            if nonce.is_some() {
                                return Err(serde::de::Error::duplicate_field("nonce"));
                            }
                            let ipld: Ipld = map.next_value()?;
                            let v = match ipld {
                                Ipld::Bytes(b) => b,
                                _ => {
                                    return Err(serde::de::Error::custom(
                                        "nonce field must be bytes",
                                    ));
                                }
                            };
                            nonce = Some(if v.len() == 16 {
                                Nonce::Nonce16(v.try_into().map_err(|e| {
                                    serde::de::Error::custom(format!(
                                        "invalid nonce bytes: {:?}",
                                        e
                                    ))
                                })?)
                            } else {
                                Nonce::Custom(v)
                            });
                        }
                        other => {
                            // Unknown field → store in `meta` as IPLD
                            // If the key already exists in meta, last one wins.
                            let val: Ipld = map.next_value()?;
                            meta.insert(other.to_owned(), val);
                        }
                    }
                }

                // Required fields:
                let issuer = issuer.ok_or_else(|| serde::de::Error::missing_field("iss"))?;
                let audience = audience.ok_or_else(|| serde::de::Error::missing_field("aud"))?;
                let subject = subject.ok_or_else(|| serde::de::Error::missing_field("sub"))?;
                let command = command.ok_or_else(|| serde::de::Error::missing_field("cmd"))?;
                let policy = policy.ok_or_else(|| serde::de::Error::missing_field("pol"))?;
                let nonce = nonce.ok_or_else(|| serde::de::Error::missing_field("nonce"))?;

                Ok(DelegationPayload {
                    issuer,
                    audience,
                    subject,
                    command,
                    policy,
                    expiration: expiration.unwrap_or(None),
                    not_before: not_before.unwrap_or(None),
                    meta,
                    nonce,
                })
            }
        }

        deserializer.deserialize_map(PayloadVisitor::<D>(std::marker::PhantomData))
    }
}

#[cfg(test)]
mod tests {
    use crate::did::{Ed25519Did, Ed25519Signer};

    use super::*;
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
            Vec<String>,
        > = DelegationBuilder::new()
            .issuer(iss.clone())
            .audience(aud)
            .subject(DelegatedSubject::Specific(sub))
            .command(vec!["read".to_string(), "write".to_string()]);

        let delegation = builder.try_build()?;

        assert_eq!(delegation.issuer().to_string(), iss.to_string());
        Ok(())
    }
}
