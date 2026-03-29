//! Signature configuration.

pub mod ecdsa;
pub mod eddsa;
pub mod web_crypto;

/// The most common signature types used in most contexts.
#[cfg(feature = "common")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Common {
    /// ES256 signature type
    Es256(ecdsa::Es256),

    /// ES256K signature type
    Es256k(ecdsa::Es256k),

    /// Ed25519 signature type
    Ed25519(eddsa::Ed25519),
}
