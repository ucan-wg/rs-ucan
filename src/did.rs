use did_url::DID;
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
/// A [Decentralized Identifier (DID)][wiki]
///
/// This is a newtype wrapper around the [`DID`] type from the [`did_url`] crate.
///
/// # Examples
///
/// ```rust
/// # use ucan::did::Did;
/// #
/// let did = Did::try_from("did:example:123".to_string()).unwrap();
/// assert_eq!(did.0.method(), "example");
/// ```
///
/// [wiki]: https://en.wikipedia.org/wiki/Decentralized_identifier
pub struct Did(pub DID);

impl From<Did> for String {
    fn from(did: Did) -> Self {
        did.0.to_string()
    }
}

impl TryFrom<String> for Did {
    type Error = <DID as TryFrom<String>>::Error;

    fn try_from(string: String) -> Result<Self, Self::Error> {
        DID::parse(&string).map(Did)
    }
}

impl fmt::Display for Did {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

impl From<Did> for Ipld {
    fn from(did: Did) -> Self {
        did.into()
    }
}

impl TryFrom<Ipld> for Did {
    type Error = FromIpldError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::String(string) => Did::try_from(string).map_err(FromIpldError::StructuralError),
            other => Err(FromIpldError::NotAnIpldString(other)),
        }
    }
}

/// Errors that can occur when converting to or from a [`Did`]
#[derive(Debug, Clone, PartialEq, Error)]
pub enum FromIpldError {
    /// Strutural errors in the [`Did`]
    StructuralError(did_url::Error),

    /// The [`Ipld`] was not a string
    NotAnIpldString(Ipld),
}

impl fmt::Display for FromIpldError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FromIpldError::StructuralError(e) => write!(f, "DID Error: {}", e),
            FromIpldError::NotAnIpldString(_ipld) => write!(f, "Not an IPLD String"), // FIXME include the bad ipld, but needs a Display instance
        }
    }
}

impl Serialize for FromIpldError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for FromIpldError {
    fn deserialize<D>(deserializer: D) -> Result<FromIpldError, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let ipld = Ipld::deserialize(deserializer)?;
        Ok(FromIpldError::NotAnIpldString(ipld))
    }
}
