use anyhow::{anyhow, Result};
use did_key::KeyPair;

use super::SigningKey;

#[cfg(feature = "rsa_support")]
use super::rsa::RsaKeyPair;

pub enum SigningKeyResult {
    Ed25519(KeyPair),

    #[cfg(feature = "rsa_support")]
    Rsa(RsaKeyPair),
}

pub fn did_to_signing_key(did: String) -> Result<SigningKeyResult> {
    if !did.starts_with("did:key:") {
        return Err(anyhow!("String is not a valid DID key: {}", did));
    }

    #[cfg(feature = "rsa_support")]
    {
        match RsaKeyPair::try_from_did(did.clone()) {
            Ok(keypair) => return Ok(SigningKeyResult::Rsa(keypair)),
            _ => (),
        };
    }

    KeyPair::try_from_did(did).map(|keypair| SigningKeyResult::Ed25519(keypair))
}
