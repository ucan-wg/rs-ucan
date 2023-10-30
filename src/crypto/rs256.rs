//! RS256 signature support

#[cfg(feature = "rs256-verifier")]
use anyhow::anyhow;
#[cfg(feature = "rs256-verifier")]
use signature::Verifier;

use super::JWSSignature;

impl JWSSignature for rsa::pkcs1v15::Signature {
    const ALGORITHM: &'static str = "RS256";
}

/// A verifier for RS256 signatures
#[cfg(feature = "rs256-verifier")]
pub fn rs256_verifier(key: &[u8], payload: &[u8], signature: &[u8]) -> Result<(), anyhow::Error> {
    let key = rsa::pkcs1::DecodeRsaPublicKey::from_pkcs1_der(key)
        .map_err(|e| anyhow!("invalid PKCS#1 key, {}", e))?;

    let key = rsa::pkcs1v15::VerifyingKey::<rsa::sha2::Sha256>::new(key);

    let signature = rsa::pkcs1v15::Signature::try_from(signature)
        .map_err(|e| anyhow!("invalid RSASSA-PKCS1-v1_5 signature, {}", e))?;

    key.verify(payload, &signature)
        .map_err(|e| anyhow!("signature mismatch, {}", e))
}
