//! `EdDSA` signature algorithms.

use crate::{
    curve::Edwards25519,
    hash::{Multihasher, Sha2_512},
    signer::Sign,
    verify::Verify,
};
use std::marker::PhantomData;

/// The `EdDSA` signature algorithm.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub struct EdDsa<C: EdDsaCurve, H: Multihasher>(PhantomData<(C, H)>);

impl<C: EdDsaCurve, H: Multihasher> EdDsa<C, H> {
    /// Create a new `EdDsa` instance.
    #[must_use]
    pub const fn new() -> Self {
        EdDsa(PhantomData)
    }
}

/// The EdDSA-compatible curves
pub trait EdDsaCurve: Sized {}
impl EdDsaCurve for Edwards25519 {}

// TODO waiting on ed448_goldilocks to cut a stable release with signing
// impl EdDsaCurve for Edwards448 {}

/// The Ed25519 signature algorithm.
///
/// The `EdDSA` signing algorithm with the Edwards25519 curve with SHA2-512 hashing.
#[cfg(all(feature = "edwards25519", feature = "sha2_512"))]
pub type Ed25519 = EdDsa<Edwards25519, Sha2_512>;

impl Verify for Ed25519 {
    type Signature = ed25519_dalek::Signature;
    type Verifier = ed25519_dalek::VerifyingKey;

    fn prefix(&self) -> u64 {
        0xed
    }

    fn config_tags(&self) -> Vec<u64> {
        vec![0xed, 0x13]
    }

    fn try_from_tags(bytes: &[u64]) -> Option<(Self, &[u64])> {
        if bytes[0..=2] == [0xed, 0xed, 0x13] {
            Some((EdDsa(PhantomData), &bytes[3..]))
        } else {
            None
        }
    }
}

impl Sign for Ed25519 {
    type Signer = ed25519_dalek::SigningKey;
    type SignError = signature::Error;
}
