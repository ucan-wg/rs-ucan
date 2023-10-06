//! EdDSA signature support

use anyhow::anyhow;
use signature::Verifier;

use super::JWSSignature;

impl JWSSignature for ed25519::Signature {
    const ALGORITHM: &'static str = "EdDSA";
}

/// A verifier for Ed25519 signatures using the `ed25519-dalek` crate
pub fn eddsa_verifier(key: &[u8], payload: &[u8], signature: &[u8]) -> Result<(), anyhow::Error> {
    let key = ed25519_dalek::VerifyingKey::try_from(key)
        .map_err(|e| anyhow!("invalid Ed25519 key, {}", e))?;

    let signature = ed25519_dalek::Signature::try_from(signature)
        .map_err(|e| anyhow!("invalid Ed25519 signature, {}", e))?;

    key.verify(payload, &signature)
        .map_err(|e| anyhow!("signature mismatch, {}", e))
}
