use anyhow::Result;
use async_trait::async_trait;

#[cfg(not(target_arch = "wasm32"))]
pub trait KeyMaterialConditionalSendSync: Send + Sync {}

#[cfg(not(target_arch = "wasm32"))]
impl<K> KeyMaterialConditionalSendSync for K where K: KeyMaterial + Send + Sync {}

#[cfg(target_arch = "wasm32")]
pub trait KeyMaterialConditionalSendSync {}

#[cfg(target_arch = "wasm32")]
impl<K> KeyMaterialConditionalSendSync for K where K: KeyMaterial {}

/// This trait must be implemented by a struct that encapsulates cryptographic
/// keypair data. The trait represent the minimum required API capability for
/// producing a signed UCAN from a cryptographic keypair, and verifying such
/// signatures.
#[cfg_attr(not(target_arch = "wasm32"), async_trait)]
#[cfg_attr(target_arch = "wasm32", async_trait(?Send))]
pub trait KeyMaterial: KeyMaterialConditionalSendSync {
    /// The algorithm that will be used to produce the signature returned by the
    /// sign method in this implementation
    fn get_jwt_algorithm_name(&self) -> String;

    /// Provides a valid DID that can be used to solve the key
    async fn get_did(&self) -> Result<String>;

    /// Sign some data with this key
    async fn sign(&self, payload: &[u8]) -> Result<Vec<u8>>;

    /// Verify the alleged signature of some data against this key
    async fn verify(&self, payload: &[u8], signature: &[u8]) -> Result<()>;
}
