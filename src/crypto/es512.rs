//! ES512 signature support

use super::JWSSignature;

impl JWSSignature for ecdsa::Signature<p521::NistP521> {
    const ALGORITHM: &'static str = "ES512";
}
