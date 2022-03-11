#[cfg(feature = "rsa_support")]
pub mod rsa;

pub mod did;

use anyhow::{anyhow, Result};
pub use did_key::{CoreSign, Ed25519KeyPair, Fingerprint, Generate, KeyPair};

/// This trait must be implemented by a struct that encapsulates cryptographic
/// keypair data. It depends on traits from the did-key crate, which are
/// republished in this module. Together, the traits represent the minimum
/// required API capability for producing a signed UCAN from a cryptographic
/// keypair.
pub trait SigningKey: CoreSign + Fingerprint + Sized {
    fn try_from_did(did: String) -> Result<Self>;

    fn get_jwt_algorithm_name(&self) -> String;

    fn get_did(&self) -> String {
        format!("did:key:{}", self.fingerprint())
    }
}

impl SigningKey for KeyPair {
    fn get_jwt_algorithm_name(&self) -> String {
        match self {
            KeyPair::Ed25519(_) => "EdDSA".into(),
            _ => "UNSUPPORTED".into(),
        }
    }

    fn try_from_did(did: String) -> Result<Self> {
        did_key::resolve(&did).map_err(|_| anyhow!("Failed to parse DID: {}", did))
    }
}

/// Verify an alleged signature of some data given a DID
pub fn verify_signature<K: SigningKey>(data: &Vec<u8>, signature: &Vec<u8>, key: &K) -> Result<()> {
    key.verify(data, signature)
        .map_err(|error| anyhow!("Failed to verify signature: {:?}", error))
}
