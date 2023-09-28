//! Cryptography utilities

use signature::SignatureEncoding;

/// A trait for mapping a SignatureEncoding to its algorithm name under JWS
pub trait JWSSignature: SignatureEncoding {
    /// The algorithm name under JWS
    // I'd originally referenced JWA types directly here, but supporting
    // unspecified algorithms, like BLS, means leaving things more open-ended.
    const ALGORITHM: &'static str;
}
