use crate::crypto::{did::KeyConstructorSlice, KeyMaterial};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use did_key::{CoreSign, Ed25519KeyPair, Fingerprint, KeyPair};

pub const SUPPORTED_KEYS: &KeyConstructorSlice = &[
    // https://github.com/multiformats/multicodec/blob/e9ecf587558964715054a0afcc01f7ace220952c/table.csv#L94
    ([0xed, 0x01], bytes_to_ed25519_key),
];

pub fn bytes_to_ed25519_key(bytes: Vec<u8>) -> Result<Box<dyn KeyMaterial>> {
    Ok(Box::new(KeyPair::Ed25519(Ed25519KeyPair::from_public_key(
        bytes.as_slice(),
    ))))
}

#[cfg_attr(feature = "web", async_trait(?Send))]
#[cfg_attr(not(feature = "web"), async_trait)]
impl KeyMaterial for KeyPair {
    fn get_jwt_algorithm_name(&self) -> String {
        "EdDSA".into()
    }

    async fn get_did(&self) -> Result<String> {
        Ok(format!("did:key:{}", self.fingerprint()))
    }

    async fn sign(&self, payload: &[u8]) -> Result<Vec<u8>> {
        Ok(CoreSign::sign(self, payload))
    }

    async fn verify(&self, payload: &[u8], signature: &[u8]) -> Result<()> {
        CoreSign::verify(self, payload, signature).map_err(|error| anyhow!("{:?}", error))
    }
}
