//! Cryptography utilities

use signature::{SignatureEncoding, Signer};

pub mod bls;
pub mod eddsa;
pub mod es256;
pub mod es256k;
pub mod es384;
pub mod es512;
pub mod ps256;
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
pub trait SignerDid<K>: Signer<K> {
    /// The DID of the signer
    fn did(&self) -> Result<String, anyhow::Error>;
}
