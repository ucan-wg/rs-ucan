//! EdDSA signature support

#[cfg(feature = "eddsa-verifier")]
use anyhow::anyhow;
#[cfg(feature = "eddsa-verifier")]
use signature::Verifier;

use multibase::Base;

use super::{JWSSignature, SignerDid};

impl JWSSignature for ed25519::Signature {
    const ALGORITHM: &'static str = "EdDSA";
}

impl SignerDid for ed25519_dalek::SigningKey {
    fn did(&self) -> Result<String, anyhow::Error> {
        let mut buf = unsigned_varint::encode::u128_buffer();
        let multicodec = unsigned_varint::encode::u128(0xed, &mut buf);

        Ok(format!(
            "did:key:{}",
            multibase::encode(
                Base::Base58Btc,
                [multicodec, self.verifying_key().to_bytes().as_ref()].concat()
            )
        ))
    }
}

/// A verifier for Ed25519 signatures using the `ed25519-dalek` crate
#[cfg(feature = "eddsa-verifier")]
pub fn eddsa_verifier(key: &[u8], payload: &[u8], signature: &[u8]) -> Result<(), anyhow::Error> {
    let key = ed25519_dalek::VerifyingKey::try_from(key)
        .map_err(|e| anyhow!("invalid Ed25519 key, {}", e))?;

    let signature = ed25519_dalek::Signature::try_from(signature)
        .map_err(|e| anyhow!("invalid Ed25519 signature, {}", e))?;

    key.verify(payload, &signature)
        .map_err(|e| anyhow!("signature mismatch, {}", e))
}
