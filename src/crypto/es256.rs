//! ES256 signature support

use super::JWSSignature;

impl JWSSignature for ecdsa::Signature<p256::NistP256> {
    const ALGORITHM: &'static str = "ES256";
}
