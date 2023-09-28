//! Cryptography utilities

use signature::SignatureEncoding;

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
