//! WebCrypto-compatible signature types and verifiers.
//!
//! This module provides composite enum types that wrap the individual
//! signature algorithms supported by the [Web Crypto API][webcrypto]:
//! ES256, ES384, ES512, and Ed25519.
//!
//! [webcrypto]: https://developer.mozilla.org/en-US/docs/Web/API/SubtleCrypto

#[cfg(feature = "web_crypto")]
use crate::{
    signature::{ecdsa, eddsa},
    verify::Verify,
};

#[cfg(feature = "web_crypto")]
use signature::{Error, SignatureEncoding, Verifier};

/// WebCrypto-compatible signature algorithm configuration.
///
/// Each variant carries the Varsig header metadata for the corresponding
/// algorithm so that [`Verify::prefix`], [`Verify::config_tags`], and
/// [`Verify::try_from_tags`] can dispatch at runtime.
#[cfg(feature = "web_crypto")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WebCrypto {
    /// ECDSA with P-256 and SHA-256
    Es256(ecdsa::Es256),

    /// ECDSA with P-384 and SHA-384
    Es384(ecdsa::Es384),

    /// ECDSA with P-521 and SHA-512
    Es512(ecdsa::Es512),

    /// `EdDSA` with `Curve25519`
    Ed25519(eddsa::Ed25519),
}

/// A signature produced by one of the `WebCrypto` algorithms.
///
/// Wraps the concrete signature type for each variant so that a single
/// enum can be passed through the [`Verify::try_verify`] pipeline.
#[cfg(feature = "web_crypto")]
#[derive(Debug, Clone, Copy)]
pub enum WebCryptoSignature {
    /// ECDSA P-256 signature
    Es256(p256::ecdsa::Signature),

    /// ECDSA P-384 signature
    Es384(p384::ecdsa::Signature),

    /// ECDSA P-521 signature
    Es512(p521::ecdsa::Signature),

    /// Ed25519 signature
    Ed25519(ed25519_dalek::Signature),
}

/// A verifying key for one of the `WebCrypto` algorithms.
///
/// The `P-521` variant uses [`ecdsa::P521VerifyingKey`] because the upstream
/// `p521` crate does not implement [`Debug`] on its verifying key type.
#[cfg(feature = "web_crypto")]
#[derive(Debug, Clone)]
pub enum WebCryptoVerifier {
    /// ECDSA P-256 verifying key
    Es256(p256::ecdsa::VerifyingKey),

    /// ECDSA P-384 verifying key
    Es384(p384::ecdsa::VerifyingKey),

    /// ECDSA P-521 verifying key (newtype for Debug)
    Es512(ecdsa::P521VerifyingKey),

    /// Ed25519 verifying key
    Ed25519(ed25519_dalek::VerifyingKey),
}

// ---------------------------------------------------------------------------
// WebCryptoSignature: SignatureEncoding
// ---------------------------------------------------------------------------

/// Canonical byte representation: we use a boxed slice because the
/// underlying signatures have different fixed sizes.
#[cfg(feature = "web_crypto")]
impl SignatureEncoding for WebCryptoSignature {
    type Repr = Box<[u8]>;
}

#[cfg(feature = "web_crypto")]
impl TryFrom<&[u8]> for WebCryptoSignature {
    type Error = Error;

    /// Attempt to decode a signature from raw bytes.
    ///
    /// Without additional context (e.g. which algorithm produced the
    /// bytes) this is ambiguous. We try each format in order.
    fn try_from(bytes: &[u8]) -> Result<Self, Self::Error> {
        // Ed25519: always 64 bytes
        if bytes.len() == ed25519_dalek::SIGNATURE_LENGTH {
            if let Ok(sig) = ed25519_dalek::Signature::from_slice(bytes) {
                return Ok(Self::Ed25519(sig));
            }
        }

        // P-256
        if let Ok(sig) = p256::ecdsa::Signature::try_from(bytes) {
            return Ok(Self::Es256(sig));
        }

        // P-384
        if let Ok(sig) = p384::ecdsa::Signature::try_from(bytes) {
            return Ok(Self::Es384(sig));
        }

        // P-521
        if let Ok(sig) = p521::ecdsa::Signature::try_from(bytes) {
            return Ok(Self::Es512(sig));
        }

        Err(Error::new())
    }
}

#[cfg(feature = "web_crypto")]
impl TryFrom<WebCryptoSignature> for Box<[u8]> {
    type Error = Error;

    fn try_from(sig: WebCryptoSignature) -> Result<Self, Self::Error> {
        match sig {
            WebCryptoSignature::Es256(s) => Ok(SignatureEncoding::to_vec(&s).into_boxed_slice()),
            WebCryptoSignature::Es384(s) => Ok(SignatureEncoding::to_vec(&s).into_boxed_slice()),
            WebCryptoSignature::Es512(s) => Ok(SignatureEncoding::to_vec(&s).into_boxed_slice()),
            WebCryptoSignature::Ed25519(s) => Ok(s.to_bytes().to_vec().into_boxed_slice()),
        }
    }
}

// ---------------------------------------------------------------------------
// WebCryptoVerifier: Verifier<WebCryptoSignature>
// ---------------------------------------------------------------------------

#[cfg(feature = "web_crypto")]
impl Verifier<WebCryptoSignature> for WebCryptoVerifier {
    fn verify(&self, msg: &[u8], signature: &WebCryptoSignature) -> Result<(), Error> {
        match (self, signature) {
            (Self::Es256(vk), WebCryptoSignature::Es256(sig)) => vk.verify(msg, sig),
            (Self::Es384(vk), WebCryptoSignature::Es384(sig)) => vk.verify(msg, sig),
            (Self::Es512(vk), WebCryptoSignature::Es512(sig)) => vk.verify(msg, sig),
            (Self::Ed25519(vk), WebCryptoSignature::Ed25519(sig)) => vk.verify(msg, sig),
            _ => Err(Error::new()),
        }
    }
}

// ---------------------------------------------------------------------------
// WebCrypto: Verify
// ---------------------------------------------------------------------------

#[cfg(feature = "web_crypto")]
impl Verify for WebCrypto {
    type Signature = WebCryptoSignature;
    type Verifier = WebCryptoVerifier;

    fn prefix(&self) -> u64 {
        match self {
            Self::Es256(v) => v.prefix(),
            Self::Es384(v) => v.prefix(),
            Self::Es512(v) => v.prefix(),
            Self::Ed25519(v) => v.prefix(),
        }
    }

    fn config_tags(&self) -> Vec<u64> {
        match self {
            Self::Es256(v) => v.config_tags(),
            Self::Es384(v) => v.config_tags(),
            Self::Es512(v) => v.config_tags(),
            Self::Ed25519(v) => v.config_tags(),
        }
    }

    fn try_from_tags(bytes: &[u64]) -> Option<(Self, &[u64])> {
        let rest = bytes.get(3..)?;

        match *bytes.first()? {
            // ECDSA prefix
            0xec => match *bytes.get(1..=2)? {
                [0x1201, 0x15] => Some((Self::Es256(ecdsa::Es256::default()), rest)),
                [0x1202, 0x20] => Some((Self::Es384(ecdsa::Es384::default()), rest)),
                [0x1202, 0x13] => Some((Self::Es512(ecdsa::Es512::default()), rest)),
                _ => None,
            },
            // EdDSA prefix
            0xed => {
                if *bytes.get(1..=2)? != [0xed, 0x13] {
                    return None;
                }
                Some((Self::Ed25519(eddsa::Ed25519::default()), rest))
            }
            _ => None,
        }
    }
}
