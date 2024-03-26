use super::key;
use super::Did;
use did_url::DID;
use enum_as_inner::EnumAsInner;
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};

#[cfg(feature = "test_utils")]
use proptest::prelude::*;

/// The set of [`Did`] types that ship with this library ("presets").
#[derive(Debug, Clone, EnumAsInner, PartialEq, PartialOrd, Ord, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Verifier {
    /// `did:key` DIDs.
    Key(key::Verifier),
    //
    // FIXME Dns(did_url::DID),
}

impl From<Verifier> for Ipld {
    fn from(verifier: Verifier) -> Self {
        match verifier {
            Verifier::Key(verifier) => verifier.into(),
        }
    }
}

impl TryFrom<Ipld> for Verifier {
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        key::Verifier::try_from(ipld)
            .map(Verifier::Key)
            .map_err(|_| ())
    }
}

impl From<Verifier> for DID {
    fn from(verifier: Verifier) -> Self {
        match verifier {
            Verifier::Key(verifier) => verifier.into(),
        }
    }
}

#[derive(Clone, EnumAsInner)]
pub enum Signer {
    Key(key::Signer),
    // FIXME Dns(did_url::DID),
}

impl std::fmt::Debug for Signer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Signer::Key(_signer) => write!(f, "Signer::Key(HIDDEN)"),
        }
    }
}

impl Did for Verifier {
    type Signature = key::Signature;
    type Signer = self::Signer;
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

#[cfg(feature = "test_utils")]
impl Arbitrary for Verifier {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        key::Verifier::arbitrary().prop_map(Verifier::Key).boxed()
    }
}
