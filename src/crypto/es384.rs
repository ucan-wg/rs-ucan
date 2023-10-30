//! ES384 signature support

#[cfg(feature = "es384-verifier")]
use anyhow::anyhow;
#[cfg(feature = "es384-verifier")]
use signature::Verifier;

use super::JWSSignature;

impl JWSSignature for p384::ecdsa::Signature {
    const ALGORITHM: &'static str = "ES384";
}

/// A verifier for ES384 signatures
#[cfg(feature = "es384-verifier")]
pub fn es384_verifier(key: &[u8], payload: &[u8], signature: &[u8]) -> Result<(), anyhow::Error> {
    let key = p384::ecdsa::VerifyingKey::try_from(key).map_err(|_| anyhow!("invalid P-384 key"))?;

    let signature =
        p384::ecdsa::Signature::try_from(signature).map_err(|_| anyhow!("invalid P-384 key"))?;

    key.verify(payload, &signature)
        .map_err(|e| anyhow!("signature mismatch, {}", e))
}
