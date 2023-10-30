//! PS256 signature support

#[cfg(feature = "ps256-verifier")]
use anyhow::anyhow;
#[cfg(feature = "ps256-verifier")]
use signature::Verifier;

use super::JWSSignature;

impl JWSSignature for rsa::pss::Signature {
    const ALGORITHM: &'static str = "PS256";
}

/// A verifier for RS256 signatures
#[cfg(feature = "ps256-verifier")]
pub fn ps256_verifier(key: &[u8], payload: &[u8], signature: &[u8]) -> Result<(), anyhow::Error> {
    let key = rsa::pkcs1::DecodeRsaPublicKey::from_pkcs1_der(key)
        .map_err(|e| anyhow!("invalid PKCS#1 key, {}", e))?;

    let key = rsa::pss::VerifyingKey::<rsa::sha2::Sha256>::new(key);

    let signature = rsa::pss::Signature::try_from(signature)
        .map_err(|e| anyhow!("invalid RSASSA-PKCS1-v1_5 signature, {}", e))?;

    key.verify(payload, &signature)
        .map_err(|e| anyhow!("signature mismatch, {}", e))
}
