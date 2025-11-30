//! Subject of a delegation

use crate::did::Did;
use serde::{de::Deserialize, ser::Serializer, Serialize};
use std::fmt::Display;

/// The Subject of a delegation
///
/// This represents what is being delegated to be later invoked.
/// To allow for powerline delegation (a node in the auth graph
/// that is a mere proxy for ANY capability), the wildcard `Any`
/// may be used.
///
/// Since it is so powerful, only use `Any` directly if you know
/// what you're doing.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Hash)]
pub enum DelegatedSubject<D: Did> {
    /// A specific subject (recommended)
    Specific(D),

    /// A wildcard subject (specialized use case)
    Any,
}

impl<D: Did> DelegatedSubject<D> {
    /// Check that the [`DelegatedSubject`] either matches the subject, or is `Any`.
    pub fn allows(&self, subject: &D) -> bool {
        match self {
            DelegatedSubject::Specific(did) => did == subject,
            DelegatedSubject::Any => true,
        }
    }

    /// Both sides match, or one is `Any`.
    pub fn coherent(&self, other: &Self) -> bool {
        match (self, other) {
            (DelegatedSubject::Any, _) | (_, DelegatedSubject::Any) => true,
            (DelegatedSubject::Specific(did), DelegatedSubject::Specific(other_did)) => {
                did == other_did
            }
        }
    }
}

impl<D: Did> From<D> for DelegatedSubject<D> {
    fn from(subject: D) -> Self {
        DelegatedSubject::Specific(subject)
    }
}

impl<D: Did + Display> Display for DelegatedSubject<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DelegatedSubject::Specific(did) => Display::fmt(did, f),
            DelegatedSubject::Any => "Null".fmt(f),
        }
    }
}

impl<D: Did + Serialize> Serialize for DelegatedSubject<D> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            DelegatedSubject::Specific(did) => did.serialize(serializer),
            DelegatedSubject::Any => serializer.serialize_none(),
        }
    }
}

impl<'de, I: Did> Deserialize<'de> for DelegatedSubject<I> {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let value = serde_value::Value::deserialize(deserializer)?;

        if value == serde_value::Value::Option(None) {
            return Ok(DelegatedSubject::Any);
        }

        if let Ok(did) = I::deserialize(value.clone()) {
            return Ok(DelegatedSubject::Specific(did));
        }

        Err(serde::de::Error::custom("invalid subject format"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::did::Ed25519Did;
    use serde_ipld_dagcbor::{from_slice, to_vec};

    #[test]
    fn any_serializes_to_null() {
        let subject: DelegatedSubject<Ed25519Did> = DelegatedSubject::Any;
        let bytes = to_vec(&subject).unwrap();
        // CBOR null is encoded as 0xf6
        assert_eq!(bytes, vec![0xf6]);
    }

    #[test]
    fn any_deserializes_from_null() {
        // CBOR null is encoded as 0xf6
        let bytes = vec![0xf6];
        let subject: DelegatedSubject<Ed25519Did> = from_slice(&bytes).unwrap();
        assert_eq!(subject, DelegatedSubject::Any);
    }

    #[test]
    fn any_roundtrip() {
        let subject: DelegatedSubject<Ed25519Did> = DelegatedSubject::Any;
        let bytes = to_vec(&subject).unwrap();
        let decoded: DelegatedSubject<Ed25519Did> = from_slice(&bytes).unwrap();
        assert_eq!(decoded, DelegatedSubject::Any);
    }

    #[test]
    fn specific_roundtrip() {
        let key = ed25519_dalek::VerifyingKey::from_bytes(&[
            215, 90, 152, 1, 130, 177, 10, 183, 213, 75, 254, 211, 201, 100, 7, 58, 14, 225, 114,
            243, 218, 166, 35, 37, 175, 2, 26, 104, 247, 7, 81, 26,
        ])
        .unwrap();
        let did: Ed25519Did = key.into();
        let subject = DelegatedSubject::Specific(did.clone());

        let bytes = to_vec(&subject).unwrap();
        let decoded: DelegatedSubject<Ed25519Did> = from_slice(&bytes).unwrap();

        assert_eq!(decoded, DelegatedSubject::Specific(did));
    }
}
