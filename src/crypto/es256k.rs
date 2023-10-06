//! ES256K signature support

use anyhow::anyhow;
use signature::Verifier;

use super::JWSSignature;

impl JWSSignature for ecdsa::Signature<k256::Secp256k1> {
    const ALGORITHM: &'static str = "ES256K";
}

/// A verifier for ES256k signatures
pub fn es256k_verifier(key: &[u8], payload: &[u8], signature: &[u8]) -> Result<(), anyhow::Error> {
    let key =
        k256::ecdsa::VerifyingKey::try_from(key).map_err(|_| anyhow!("invalid secp256k1 key"))?;

    let signature = k256::ecdsa::Signature::try_from(signature)
        .map_err(|_| anyhow!("invalid secp256k1 key"))?;

    key.verify(payload, &signature)
        .map_err(|e| anyhow!("signature mismatch, {}", e))
}
