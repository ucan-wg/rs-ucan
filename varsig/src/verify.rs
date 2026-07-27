//! Signature verification and configuration.

use alloc::vec::Vec;
use core::{error::Error, fmt::Debug};

use crate::codec::Codec;
use signature::{SignatureEncoding, Verifier};
use thiserror::Error;

/// A trait for signature verification (e.g. public keys).
pub trait Verify: Sized + Debug {
    /// The signature type for the header.
    type Signature: SignatureEncoding + Debug;

    /// The associated signer (referenced or owned signing key for the header).
    type Verifier: Verifier<Self::Signature> + Debug;

    /// The prefix for the signature type.
    ///
    /// For example, `EdDSA` would be `0xED`.
    fn prefix(&self) -> u64;

    /// The configuration as a series of [`u64`] tags.
    ///
    /// NOTE: these will be automatically converted to LEB128 by the serializer.
    fn config_tags(&self) -> Vec<u64>;

    /// Try to create a codec from a series of bytes.
    fn try_from_tags(bytes: &[u64]) -> Option<(Self, &[u64])>;

    /// Try to verify a signature for some payload.
    ///
    /// This method encodes the payload using the provided codec,
    /// then verifies the signature. The payload does not need to be
    /// serialized ahead of time (the `codec` field configures that).
    ///
    /// # Errors
    ///
    /// If the encoding fails, it returns an error of type `VerificationError::EncodingError`.
    /// If the verification fails, it returns an error of type `VerificationError::VerificationError`.
    fn try_verify<T, C: Codec<T>>(
        &self,
        codec: &C,
        verifier: &Self::Verifier, // e.g. verifying ("public") key
        signature: &Self::Signature,
        payload: &T,
    ) -> Result<(), VerificationError<C::EncodingError>> {
        let buffer = codec
            .encode_payload(payload)
            .map_err(VerificationError::EncodingError)?;
        verifier
            .verify(&buffer, signature)
            .map_err(VerificationError::VerificationError)
    }
}

/// Error type for verification errors.
#[derive(Debug, Error)]
pub enum VerificationError<E: Error> {
    /// Codec error.
    #[error(transparent)]
    EncodingError(E),

    /// Verification error.
    #[error("Verification error: {0}")]
    VerificationError(signature::Error),
}
