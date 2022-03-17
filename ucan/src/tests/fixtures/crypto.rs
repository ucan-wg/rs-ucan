use crate::crypto::{did::KeyConstructorSlice, SigningKey};
use anyhow::{anyhow, Result};
use did_key::{CoreSign, Ed25519KeyPair, Fingerprint, KeyPair};

pub const SUPPORTED_KEYS: &KeyConstructorSlice = &[
    // https://github.com/multiformats/multicodec/blob/e9ecf587558964715054a0afcc01f7ace220952c/table.csv#L94
    ([0xed, 0x01], bytes_to_ed25519_key),
];

pub fn bytes_to_ed25519_key(bytes: Vec<u8>) -> Box<dyn SigningKey> {
    Box::new(KeyPair::Ed25519(Ed25519KeyPair::from_public_key(
        bytes.as_slice(),
    )))
}

impl SigningKey for KeyPair {
    fn get_jwt_algorithm_name(&self) -> String {
        "EdDSA".into()
    }

    fn get_did(&self) -> String {
        format!("did:key:{}", self.fingerprint())
    }

    fn sign(&self, payload: &[u8]) -> Result<Vec<u8>> {
        Ok(CoreSign::sign(self, payload))
    }

    fn verify(&self, payload: &[u8], signature: &[u8]) -> Result<()> {
        CoreSign::verify(self, payload, signature).map_err(|error| anyhow!("{:?}", error))
    }
}
