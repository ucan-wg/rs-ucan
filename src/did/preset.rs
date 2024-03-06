use super::key;
use super::Did;
use did_url::DID;
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

impl From<Verifier> for DID {
    fn from(verifier: Verifier) -> Self {
        match verifier {
            Verifier::Key(verifier) => verifier.into(),
        }
    }
}

#[derive(Debug, Clone, EnumAsInner, PartialEq, Eq)]
pub enum Signer {
    Key(key::Signer),
    // FIXME Dns(did_url::DID),
}

impl Did for Verifier {
    type Signature = key::Signature;
    type Signer = Signer;
}

impl TryFrom<DID> for Verifier {
    type Error = key::FromStrError;

    fn try_from(did: DID) -> Result<Self, Self::Error> {
        key::Verifier::try_from(did).map(Verifier::Key)
    }
}

impl signature::Signer<key::Signature> for Signer {
    fn try_sign(&self, message: &[u8]) -> Result<key::Signature, signature::Error> {
        match self {
            Signer::Key(signer) => signer.try_sign(message),
        }
    }
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
