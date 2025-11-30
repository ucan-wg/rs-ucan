//! UCAN Invocation
//!
//! The spec for UCAN Invocations can be found at
//! [the GitHub repo](https://github.com/ucan-wg/invocation/).

pub mod builder;

use crate::{
    cid::to_dagcbor_cid,
    command::Command,
    crypto::nonce::Nonce,
    delegation::{
        policy::predicate::{Predicate, RunError},
        store::DelegationStore,
    },
    did::{Did, DidSigner},
    envelope::{payload_tag::PayloadTag, Envelope},
    future::FutureKind,
    promise::{Promised, WaitingOn},
    time::timestamp::Timestamp,
    unset::Unset,
    Delegation,
};
use builder::InvocationBuilder;
use ipld_core::{cid::Cid, ipld::Ipld};
use serde::{Deserialize, Serialize};
use std::{borrow::Borrow, collections::BTreeMap, fmt::Debug};
use thiserror::Error;
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
    pub const fn command(&self) -> &Command {
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

    // FIXME delegation store trait
    // pub fn check(&self, proof_store: BTreeMap<Cid, Verified<Delegation<D>>> -> Result<(), ()> {
    //     self.try_verify(self.codec(), self.verifier(), self.signature(), self.payload())?;

    //     let mut realized_proofs = Vec::new();
    //     for proof_cid in self.proofs() {
    //         let found = proof_store.get(proof_cid).expect("FIXME");
    //         realized_proofs.push(found);
    //     }
    //     todo!("FIXME")
    // }
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
    pub(crate) command: Command,

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
    pub const fn command(&self) -> &Command {
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

    /// Compute the CID for this invocation.
    pub fn to_cid(&self) -> Cid {
        to_dagcbor_cid(&self)
    }

    /// Check if an [`InvocationPayload`] with proofs stored in a delegation store is valid.
    ///
    /// # Errors
    ///
    /// Returns a [`StoredCheckError`] if the check fails.
    pub async fn check<K: FutureKind, T: Borrow<Delegation<D>>, S: DelegationStore<K, D, T>>(
        &self,
        proof_store: &S,
    ) -> Result<(), StoredCheckError<K, D, T, S>> {
        let realized_proofs: Vec<T> = proof_store
            .get_all(&self.proofs)
            .await
            .map_err(StoredCheckError::GetError)?;
        let dlgs: Vec<&Delegation<D>> = realized_proofs.iter().map(Borrow::borrow).collect();
        self.syntatic_checks(dlgs)?;
        Ok(())
    }

    /// Check if an [`InvocationPayload`] is valid.
    ///
    /// # Errors
    ///
    /// Returns a [`CheckFailed`] if the check fails.
    pub fn syntatic_checks<'a, I: IntoIterator<Item = &'a Delegation<D>>>(
        &'a self,
        proofs: I,
    ) -> Result<(), CheckFailed> {
        let args: Ipld = self
            .arguments()
            .iter()
            .map(|(k, v)| v.try_into().map(|ipld| (k.clone(), ipld)))
            .collect::<Result<BTreeMap<String, Ipld>, _>>()?
            .into();

        let mut expected_issuer = self.subject();

        for proof in proofs {
            if proof.subject().allows(self.subject()) {
                return Err(CheckFailed::SubjectNotAllowedByProof);
            }

            if proof.issuer() != expected_issuer {
                return Err(CheckFailed::InvalidProofIssuerChain);
            }

            if !self.command.starts_with(proof.command()) {
                return Err(CheckFailed::CommandMismatch {
                    found: proof.command().clone(),
                    expected: self.command.clone(),
                });
            }

            for predicate in proof.policy() {
                if !predicate.clone().run(&args)? {
                    return Err(CheckFailed::PredicateFailed(Box::new(predicate.clone())));
                }
            }

            expected_issuer = proof.audience();
        }

        if expected_issuer != self.issuer() {
            return Err(CheckFailed::InvalidProofIssuerChain);
        }

        Ok(())
    }
}

impl<D: Did> PayloadTag for InvocationPayload<D> {
    fn spec_id() -> &'static str {
        "inv"
    }

    fn version() -> &'static str {
        "1.0.0-rc.1"
    }
}

/// Errors that can occur when checking an invocation
#[derive(Debug, Clone, Error)]
pub enum CheckFailed {
    /// Error indicating that the invocation is waiting on a promise to be resolved
    #[error(transparent)]
    WaitingOnPromise(#[from] WaitingOn),

    /// Error indicating that the command in the invocation does not match the command in the proof
    #[error("command mismatch: expected {expected:?}, found {expected:?}")]
    CommandMismatch {
        /// The expected command
        expected: Command,

        /// The found command
        found: Command,
    },
    /// Error indicating that a predicate failed to run
    #[error(transparent)]
    PredicateRunError(#[from] RunError),

    /// Error indicating that a predicate has failed
    #[error("predicate failed: {0:?}")]
    PredicateFailed(Box<Predicate>),

    /// Error indicating that the proof issuer chain is invalid
    #[error("invalid proof issuer chain")]
    InvalidProofIssuerChain,

    /// Error indicating that the invocation's subject is not allowed by the proof's subject
    #[error("subject not allowed by proof")]
    SubjectNotAllowedByProof,

    /// Error indicating that the root proof's issuer is not the same as the invocation's subject
    #[error("root proof issuer is not the subject")]
    RootProofIssuerIsNotSubject,
}

/// Errors that can occur when checking an invocation with proofs stored in a delegation store
#[derive(Debug, Clone, Error)]
pub enum StoredCheckError<
    K: FutureKind,
    D: Did,
    T: Borrow<Delegation<D>>,
    S: DelegationStore<K, D, T>,
> {
    /// Error getting proofs from the store
    #[error(transparent)]
    GetError(S::GetError),

    /// Proof check failed
    #[error(transparent)]
    CheckFailed(#[from] CheckFailed),
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
            Command,
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
