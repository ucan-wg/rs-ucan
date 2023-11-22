//! Cryptography utilities

use signature::SignatureEncoding;

#[cfg(feature = "bls")]
pub mod bls;
#[cfg(feature = "eddsa")]
pub mod eddsa;
#[cfg(feature = "es256")]
pub mod es256;
#[cfg(feature = "es256k")]
pub mod es256k;
#[cfg(feature = "es384")]
pub mod es384;
#[cfg(feature = "es512")]
pub mod es512;
#[cfg(feature = "ps256")]
pub mod ps256;
#[cfg(feature = "rs256")]
pub mod rs256;

/// A trait for mapping a SignatureEncoding to its algorithm name under JWS
pub trait JWSSignature: SignatureEncoding {
    /// The algorithm name under JWS
    // I'd originally referenced JWA types directly here, but supporting
    // unspecified algorithms, like BLS, means leaving things more open-ended.
    const ALGORITHM: &'static str;
}

/// A trait for mapping a Signer<K> to its DID. In most cases, this will
/// be a DID with method did-key, however other methods can be supported
/// by implementing this trait for a custom signer.
pub trait SignerDid {
    /// The DID of the signer
    fn did(&self) -> Result<String, anyhow::Error>;
}
