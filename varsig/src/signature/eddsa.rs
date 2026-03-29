//! `EdDSA` signature algorithms.

use core::marker::PhantomData;

use crate::hash::Multihasher;

#[cfg(all(feature = "edwards25519", feature = "sha2_512"))]
use alloc::{vec, vec::Vec};

#[cfg(all(feature = "edwards25519", feature = "sha2_512"))]
use crate::verify::Verify;

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

#[cfg(feature = "edwards25519")]
impl EdDsaCurve for crate::curve::Edwards25519 {}

/// The Ed25519 signature algorithm.
///
/// The `EdDSA` signing algorithm with the Edwards25519 curve with SHA2-512 hashing.
#[cfg(all(feature = "edwards25519", feature = "sha2_512"))]
pub type Ed25519 = EdDsa<crate::curve::Edwards25519, crate::hash::Sha2_512>;

#[cfg(all(feature = "edwards25519", feature = "sha2_512"))]
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
        if *bytes.get(0..=2)? == [0xed, 0xed, 0x13] {
            Some((EdDsa(PhantomData), bytes.get(3..)?))
        } else {
            None
        }
    }
}

#[cfg(all(feature = "edwards25519", feature = "sha2_512"))]
impl crate::signer::Sign for Ed25519 {
    type Signer = ed25519_dalek::SigningKey;
    type SignError = signature::Error;
}
