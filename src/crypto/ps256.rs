//! PS256 signature support

use anyhow::anyhow;
use signature::Verifier;

use super::JWSSignature;

impl JWSSignature for rsa::pss::Signature {
    const ALGORITHM: &'static str = "PS256";
}

/// A verifier for PS256 signatures
pub fn p256_verifier(key: &[u8], payload: &[u8], signature: &[u8]) -> Result<(), anyhow::Error> {
    let key = p256::ecdsa::VerifyingKey::try_from(key).map_err(|_| anyhow!("invalid P-256 key"))?;

    let signature =
        p256::ecdsa::Signature::try_from(signature).map_err(|_| anyhow!("invalid P-256 key"))?;

    key.verify(payload, &signature)
        .map_err(|e| anyhow!("signature mismatch, {}", e))
}
