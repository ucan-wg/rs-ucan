use std::collections::BTreeMap;

use anyhow::{anyhow, Result};

use super::KeyMaterial;

pub type DidPrefix = [u8; 2];
pub type BytesToKey = fn(Vec<u8>) -> Box<dyn KeyMaterial>;
pub type KeyConstructors = BTreeMap<DidPrefix, BytesToKey>;
pub type KeyConstructorSlice = [(DidPrefix, BytesToKey)];

pub const BASE58_DID_PREFIX: &str = "did:key:z";

/// A parser that is able to convert from a DID string into a corresponding
/// [`crypto::SigningKey`] implementation. The parser extracts the signature
/// magic bytes from a given DID and tries to match them to a corresponding
/// constructor function that produces a `SigningKey`.
pub struct DidParser {
    key_constructors: KeyConstructors,
}

impl DidParser {
    pub fn new<'a>(key_constructor_slice: &'a KeyConstructorSlice) -> Self {
        let mut key_constructors = BTreeMap::new();
        for pair in key_constructor_slice {
            key_constructors.insert(pair.0, pair.1);
        }
        DidParser { key_constructors }
    }

    pub fn parse(&self, did: String) -> Result<Box<dyn KeyMaterial>> {
        if !did.starts_with(BASE58_DID_PREFIX) {
            return Err(anyhow!("Not a DID: {}", did));
        }

        let did_bytes = bs58::decode(&did[BASE58_DID_PREFIX.len()..]).into_vec()?;
        let magic_bytes = &did_bytes[0..2];

        match self.key_constructors.get(magic_bytes) {
            Some(ctor) => Ok(ctor(Vec::from(&did_bytes[2..]))),
            None => Err(anyhow!("Unrecognized magic bytes: {:?}", magic_bytes)),
        }
    }
}
