//! ES256K signature support

#[cfg(feature = "es256k-verifier")]
use anyhow::anyhow;
#[cfg(feature = "es256k-verifier")]
use signature::Verifier;

use super::JWSSignature;

impl JWSSignature for k256::ecdsa::Signature {
    const ALGORITHM: &'static str = "ES256K";
}

/// A verifier for ES256k signatures
#[cfg(feature = "es256k-verifier")]
pub fn es256k_verifier(key: &[u8], payload: &[u8], signature: &[u8]) -> Result<(), anyhow::Error> {
    let key =
        k256::ecdsa::VerifyingKey::try_from(key).map_err(|_| anyhow!("invalid secp256k1 key"))?;

    let signature = k256::ecdsa::Signature::try_from(signature)
        .map_err(|_| anyhow!("invalid secp256k1 key"))?;

    key.verify(payload, &signature)
        .map_err(|e| anyhow!("signature mismatch, {}", e))
}
