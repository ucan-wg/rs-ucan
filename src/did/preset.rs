use super::key;
use enum_as_inner::EnumAsInner;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

/// The set of [`Did`] types that ship with this library ("presets").
#[derive(Debug, Clone, EnumAsInner, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Verifier {
    /// `did:key` DIDs.
    Key(key::Verifier),
    //
    // FIXME Dns(did_url::DID),
}

impl signature::Verifier<key::Signature> for Verifier {
    fn verify(&self, message: &[u8], signature: &key::Signature) -> Result<(), signature::Error> {
        match self {
            Verifier::Key(verifier) => verifier.verify(message, signature),
        }
    }
}

impl Display for Verifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Verifier::Key(verifier) => verifier.fmt(f),
        }
    }
}

impl FromStr for Verifier {
    type Err = key::FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        key::Verifier::from_str(s).map(Verifier::Key)
    }
}
