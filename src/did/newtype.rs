use did_url::DID;
use enum_as_inner::EnumAsInner;
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use std::{fmt, string::ToString};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// A [Decentralized Identifier (DID)][wiki]
///
/// This is a newtype wrapper around the [`DID`] type from the [`did_url`] crate.
///
/// # Examples
///
/// ```rust
/// # use ucan::did;
/// #
/// let did = did::Newtype::try_from("did:example:123".to_string()).unwrap();
/// assert_eq!(did.0.method(), "example");
/// ```
///
/// [wiki]: https://en.wikipedia.org/wiki/Decentralized_identifier
pub struct Newtype(pub DID);

impl Serialize for Newtype {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        String::from(self.clone()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Newtype {
    fn deserialize<D>(deserializer: D) -> Result<Newtype, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        Newtype::try_from(string).map_err(serde::de::Error::custom)
    }
}

impl From<Newtype> for String {
    fn from(did: Newtype) -> Self {
        did.0.to_string()
    }
}

impl TryFrom<String> for Newtype {
    type Error = <DID as TryFrom<String>>::Error;

    fn try_from(string: String) -> Result<Self, Self::Error> {
        DID::parse(&string).map(Newtype)
    }
}

impl fmt::Display for Newtype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

impl From<Newtype> for Ipld {
    fn from(did: Newtype) -> Self {
        did.into()
    }
}

impl TryFrom<Ipld> for Newtype {
    type Error = FromIpldError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::String(string) => {
                Newtype::try_from(string).map_err(FromIpldError::StructuralError)
            }
            other => Err(FromIpldError::NotAnIpldString(other)),
        }
    }
}

/// Errors that can occur when converting to or from a [`Newtype`]
#[derive(Debug, Clone, EnumAsInner, PartialEq, Error)]
pub enum FromIpldError {
    /// Strutural errors in the [`Newtype`]
    #[error(transparent)]
    StructuralError(#[from] did_url::Error),

    /// The [`Ipld`] was not a string
    #[error("Not an IPLD String")]
    NotAnIpldString(Ipld),
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
