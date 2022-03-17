pub mod did;

use anyhow::{anyhow, Result};

/// This trait must be implemented by a struct that encapsulates cryptographic
/// keypair data. The trait represent the minimum required API capability for
/// producing a signed UCAN from a cryptographic keypair, and verifying such
/// signatures.
pub trait SigningKey {
    fn get_jwt_algorithm_name(&self) -> String;
    fn get_did(&self) -> String;

    /// Sign some data with this key
    fn sign(&self, payload: &[u8]) -> Result<Vec<u8>>;

    /// Verify the alleged signature of some data against this key
    fn verify(&self, payload: &[u8], signature: &[u8]) -> Result<()>;
}

/// Verify an alleged signature of some data given a DID
pub fn verify_signature<K: SigningKey>(data: &Vec<u8>, signature: &Vec<u8>, key: &K) -> Result<()> {
    key.verify(data, signature)
        .map_err(|error| anyhow!("Failed to verify signature: {:?}", error))
}
