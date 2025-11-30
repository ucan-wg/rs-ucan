//! Decentralized Identifier (DID) helpers.

use base58::ToBase58;
use serde::{Deserialize, Deserializer, Serialize};
use std::{fmt::Debug, str::FromStr};
use thiserror::Error;
use varsig::{signature::eddsa::Ed25519, signer::Sign};

/// A trait for [DID]s.
///
/// [DID]: https://en.wikipedia.org/wiki/Decentralized_identifier
pub trait Did:
    PartialEq + ToString + FromStr + Serialize + for<'de> Deserialize<'de> + Debug
{
    /// The associated `Varsig` configuration.
    type VarsigConfig: Sign + Clone;

    /// Get the DID method header (e.g. `key` for `did-keys`)
    fn did_method(&self) -> &str;

    /// Get the associated `Varsig` configuration.
    fn varsig_config(&self) -> &Self::VarsigConfig;
}

/// A trait for DID signers.
pub trait DidSigner {
    /// The associated DID type.
    type Did: Did + Clone;

    /// Get the associated DID.
    fn did(&self) -> &Self::Did;

    /// Get the associated signer instance.
    fn signer(&self) -> &<<Self::Did as Did>::VarsigConfig as Sign>::Signer;
}

/// An `Ed25519` `did:key`.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Ed25519Did(pub ed25519_dalek::VerifyingKey, Ed25519);

impl From<ed25519_dalek::VerifyingKey> for Ed25519Did {
    fn from(key: ed25519_dalek::VerifyingKey) -> Self {
        Ed25519Did(key, Ed25519::new())
    }
}

impl From<ed25519_dalek::SigningKey> for Ed25519Did {
    fn from(key: ed25519_dalek::SigningKey) -> Self {
        let verifying_key = key.verifying_key();
        Ed25519Did(verifying_key, Ed25519::new())
    }
}

impl std::fmt::Display for Ed25519Did {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut raw_bytes = Vec::with_capacity(34);
        raw_bytes.push(0xed);
        raw_bytes.push(0x01);
        raw_bytes.extend_from_slice(self.0.as_bytes());
        let b58 = ToBase58::to_base58(raw_bytes.as_slice());
        write!(f, "did:key:z{b58}")
    }
}

impl FromStr for Ed25519Did {
    type Err = Ed25519DidFromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let parts: Vec<&str> = s.split(':').collect();
        let did_tag = *parts
            .first()
            .ok_or(Ed25519DidFromStrError::InvalidDidHeader)?;
        let key_tag = *parts
            .get(1)
            .ok_or(Ed25519DidFromStrError::InvalidDidHeader)?;

        if parts.len() != 3 || did_tag != "did" || key_tag != "key" {
            return Err(Ed25519DidFromStrError::InvalidDidHeader);
        }
        let b58 = parts
            .get(2)
            .ok_or(Ed25519DidFromStrError::InvalidDidHeader)?
            .strip_prefix('z')
            .ok_or(Ed25519DidFromStrError::MissingBase58Prefix)?;
        let key_bytes =
            base58::FromBase58::from_base58(b58).map_err(|_| Ed25519DidFromStrError::InvalidKey)?;
        let raw_arr = <[u8; 34]>::try_from(key_bytes.as_slice())
            .map_err(|_| Ed25519DidFromStrError::InvalidKey)?;
        if raw_arr[0] != 0xed || raw_arr[1] != 0x01 {
            return Err(Ed25519DidFromStrError::InvalidKey);
        }
        let key_arr: [u8; 32] = raw_arr[2..]
            .try_into()
            .map_err(|_| Ed25519DidFromStrError::InvalidKey)?;
        let key = ed25519_dalek::VerifyingKey::from_bytes(&key_arr)
            .map_err(|_| Ed25519DidFromStrError::InvalidKey)?;
        Ok(Ed25519Did(key, Ed25519::new()))
    }
}

/// Errors that can occur when parsing an `Ed25519Did` from a string.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Error)]
pub enum Ed25519DidFromStrError {
    /// The DID header is invalid.
    #[error("invalid did header")]
    InvalidDidHeader,

    /// The base58 prefix 'z' is missing.
    #[error("missing base58 prefix 'z'")]
    MissingBase58Prefix,

    /// The base58 encoding is invalid.
    #[error("invalid base58 encoding")]
    InvalidBase58,

    /// The key bytes are invalid.
    #[error("invalid key bytes")]
    InvalidKey,
}

impl Did for Ed25519Did {
    type VarsigConfig = Ed25519;

    fn did_method(&self) -> &'static str {
        "key"
    }

    fn varsig_config(&self) -> &Self::VarsigConfig {
        &self.1
    }
}

impl Serialize for Ed25519Did {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

// impl<'de> Deserialize<'de> for Ed25519Did {
//     fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
//     where
//         D: serde::Deserializer<'de>,
//     {
//         let s = String::deserialize(deserializer)?;
//         s.parse()
//             .map_err(|_| serde::de::Error::custom(format!("unable to parse did from string: {s}")))
//     }
// }
impl<'de> Deserialize<'de> for Ed25519Did {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct DidKeyVisitor;

        impl serde::de::Visitor<'_> for DidKeyVisitor {
            type Value = Ed25519Did;

            fn expecting(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str("a did:key string containing an ed25519 public key")
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: serde::de::Error,
            {
                const DID_PREFIX: &str = "did:key:z";
                const ED25519_PUB: [u8; 2] = [0xED, 0x01];

                if !v.starts_with(DID_PREFIX) {
                    return Err(E::custom("expected did:key with base58btc (did:key:z…)"));
                }

                let b58_payload = &v[DID_PREFIX.len()..];
                let decoded = base58::FromBase58::from_base58(b58_payload)
                    .map_err(|e| E::custom(format!("base58 decode failed: {e:?}")))?;

                if decoded.len() != 34 {
                    return Err(E::custom(format!(
                        "unexpected byte length: got {}, want 34 (2-byte header + 32-byte key)",
                        decoded.len()
                    )));
                }

                let leading = decoded.get(0..2).ok_or_else(|| {
                    E::custom("decoded did:key payload too short to contain multicodec header")
                })?;

                if leading != ED25519_PUB {
                    return Err(E::custom("not an ed25519-pub multicodec (0xED 0x01)"));
                }

                let remainder = decoded.get(2..).ok_or_else(|| {
                    E::custom("decoded did:key payload too short to contain ed25519 public key")
                })?;

                #[allow(clippy::expect_used)]
                let key_bytes: [u8; 32] =
                    remainder.try_into().expect("slice length verified above");

                let vk = ed25519_dalek::VerifyingKey::from_bytes(&key_bytes).map_err(|e| {
                    E::custom(format!(
                        "failed to construct ed25519 public key from bytes: {e:?}"
                    ))
                })?;

                Ok(Ed25519Did(vk, Ed25519::new()))
            }
        }

        deserializer.deserialize_str(DidKeyVisitor)
    }
}

/// An `Ed25519` `did:key` signer.
#[derive(Debug, Clone)]
pub struct Ed25519Signer {
    did: Ed25519Did,
    signer: ed25519_dalek::SigningKey,
}

impl Ed25519Signer {
    /// Create a new `Ed25519Signer` from a signing key.
    #[must_use]
    pub fn new(signer: ed25519_dalek::SigningKey) -> Self {
        let verifying_key = signer.verifying_key();
        let did = Ed25519Did(verifying_key, Ed25519::new());
        Self { did, signer }
    }

    /// Get the associated DID.
    #[must_use]
    pub const fn did(&self) -> &Ed25519Did {
        &self.did
    }

    /// Get the associated signer.
    #[must_use]
    pub const fn signer(&self) -> &ed25519_dalek::SigningKey {
        &self.signer
    }
}

impl From<ed25519_dalek::SigningKey> for Ed25519Signer {
    fn from(signer: ed25519_dalek::SigningKey) -> Self {
        Self::new(signer)
    }
}

impl std::fmt::Display for Ed25519Signer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.did)
    }
}

impl DidSigner for Ed25519Signer {
    type Did = Ed25519Did;

    fn did(&self) -> &Self::Did {
        &self.did
    }

    fn signer(&self) -> &<<Self::Did as Did>::VarsigConfig as Sign>::Signer {
        &self.signer
    }
}

impl Serialize for Ed25519Signer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.did.serialize(serializer)
    }
}
