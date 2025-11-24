//! UCAN Invocation
//!
//! The spec for UCAN Invocations can be found at
//! [the GitHub repo](https://github.com/ucan-wg/invocation/).

pub mod builder;

use crate::{
    crypto::nonce::Nonce,
    did::{Did, DidSigner},
    envelope::Envelope,
    promise::Promised,
    time::timestamp::Timestamp,
    unset::Unset,
};
use builder::InvocationBuilder;
use ipld_core::{cid::Cid, ipld::Ipld};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Debug};
use varsig::verify::Verify;

/// Top-level UCAN Invocation.
///
/// This is the token that commands the receiver to perform some action.
/// It is backed by UCAN Delegation(s).
#[derive(Clone)]
pub struct Invocation<D: Did>(
    Envelope<D::VarsigConfig, InvocationPayload<D>, <D::VarsigConfig as Verify>::Signature>,
);

impl<D: Did> Invocation<D> {
    /// Creates a blank [`InvocationBuilder`] instance.
    #[must_use]
    pub const fn builder<S: DidSigner<Did = D>>() -> InvocationBuilder<S, Unset, Unset, Unset, Unset>
    {
        InvocationBuilder::new()
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
    pub const fn subject(&self) -> &D {
        &self.0 .1.payload.subject
    }

    /// Getter for the `command` field.
    pub const fn command(&self) -> &Vec<String> {
        &self.0 .1.payload.command
    }

    /// Getter for the `arguments` field.
    pub const fn arguments(&self) -> &BTreeMap<String, Promised> {
        &self.0 .1.payload.arguments
    }

    /// Getter for the `proofs` field.
    pub const fn proofs(&self) -> &Vec<Cid> {
        &self.0 .1.payload.proofs
    }

    /// Getter for the `cause` field.
    pub const fn cause(&self) -> Option<Cid> {
        self.0 .1.payload.cause
    }

    /// Getter for the `expiration` field.
    pub const fn expiration(&self) -> Option<Timestamp> {
        self.0 .1.payload.expiration
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

impl<D: Did> Debug for Invocation<D> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Invocation").field(&self.0).finish()
    }
}

impl<D: Did> Serialize for Invocation<D> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.serialize(serializer)
    }
}

impl<'de, I: Did> Deserialize<'de> for Invocation<I>
where
    <I::VarsigConfig as Verify>::Signature: for<'xe> Deserialize<'xe>,
{
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let envelope = Envelope::<_, _, _>::deserialize(deserializer)?;
        Ok(Invocation(envelope))
    }
}

/// UCAN Invocation
///
/// Invoke a UCAN capability. This type implements the
/// [UCAN Invocation spec](https://github.com/ucan-wg/invocation/README.md).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(bound(deserialize = "D: Did"))]
pub struct InvocationPayload<D: Did> {
    #[serde(rename = "iss")]
    pub(crate) issuer: D,

    #[serde(rename = "aud")]
    pub(crate) audience: D,

    #[serde(rename = "sub")]
    pub(crate) subject: D,

    #[serde(rename = "cmd")]
    pub(crate) command: Vec<String>,

    #[serde(rename = "arg")]
    pub(crate) arguments: BTreeMap<String, Promised>,

    #[serde(rename = "prf")]
    pub(crate) proofs: Vec<Cid>,

    pub(crate) cause: Option<Cid>,

    #[serde(rename = "iat")]
    pub(crate) issued_at: Option<Timestamp>,

    #[serde(rename = "exp")]
    pub(crate) expiration: Option<Timestamp>,

    pub(crate) meta: BTreeMap<String, Ipld>,
    pub(crate) nonce: Nonce,
}

impl<D: Did> InvocationPayload<D> {
    /// Getter for the `issuer` field.
    pub const fn issuer(&self) -> &D {
        &self.issuer
    }

    /// Getter for the `audience` field.
    pub const fn audience(&self) -> &D {
        &self.audience
    }

    /// Getter for the `subject` field.
    pub const fn subject(&self) -> &D {
        &self.subject
    }

    /// Getter for the `command` field.
    pub const fn command(&self) -> &Vec<String> {
        &self.command
    }

    /// Getter for the `arguments` field.
    pub const fn arguments(&self) -> &BTreeMap<String, Promised> {
        &self.arguments
    }

    /// Getter for the `proofs` field.
    pub const fn proofs(&self) -> &Vec<Cid> {
        &self.proofs
    }

    /// Getter for the `cause` field.
    pub const fn cause(&self) -> Option<Cid> {
        self.cause
    }

    /// Getter for the `expiration` field.
    pub const fn expiration(&self) -> Option<Timestamp> {
        self.expiration
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

#[cfg(test)]
mod tests {
    use crate::{
        did::{Ed25519Did, Ed25519Signer},
        invocation::builder::InvocationBuilder,
    };

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

        let builder: InvocationBuilder<
            Ed25519Signer,
            Ed25519Signer,
            Ed25519Did,
            Ed25519Did,
            Vec<String>,
            Vec<Cid>,
        > = InvocationBuilder::new()
            .issuer(iss.clone())
            .audience(aud)
            .subject(sub)
            .command(vec!["read".to_string(), "write".to_string()])
            .proofs(vec![]);

        let invocation = builder.try_build()?;

        assert_eq!(invocation.issuer().to_string(), iss.to_string());
        Ok(())
    }
}
