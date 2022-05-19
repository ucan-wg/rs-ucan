use super::KeyMaterial;
use anyhow::{anyhow, Result};
use std::{collections::BTreeMap, sync::Arc};

pub type DidPrefix = [u8; 2];
pub type BytesToKey = fn(Vec<u8>) -> Result<Box<dyn KeyMaterial>>;
pub type KeyConstructors = BTreeMap<DidPrefix, BytesToKey>;
pub type KeyConstructorSlice = [(DidPrefix, BytesToKey)];
pub type KeyCache = BTreeMap<String, Arc<Box<dyn KeyMaterial>>>;

pub const BASE58_DID_PREFIX: &str = "did:key:z";

/// A parser that is able to convert from a DID string into a corresponding
/// [`crypto::SigningKey`] implementation. The parser extracts the signature
/// magic bytes from a given DID and tries to match them to a corresponding
/// constructor function that produces a `SigningKey`.
pub struct DidParser {
    key_constructors: KeyConstructors,
    key_cache: KeyCache,
}

impl DidParser {
    pub fn new(key_constructor_slice: &KeyConstructorSlice) -> Self {
        let mut key_constructors = BTreeMap::new();
        for pair in key_constructor_slice {
            key_constructors.insert(pair.0, pair.1);
        }
        DidParser {
            key_constructors,
            key_cache: BTreeMap::new(),
        }
    }

    pub fn parse(&mut self, did: &str) -> Result<Arc<Box<dyn KeyMaterial>>> {
        if !did.starts_with(BASE58_DID_PREFIX) {
            return Err(anyhow!("Not a DID: {}", did));
        }

        let did = did.to_owned();
        if let Some(key) = self.key_cache.get(&did) {
            return Ok(key.clone());
        }

        let did_bytes = bs58::decode(&did[BASE58_DID_PREFIX.len()..]).into_vec()?;
        let magic_bytes = &did_bytes[0..2];
        match self.key_constructors.get(magic_bytes) {
            Some(ctor) => {
                let key = ctor(Vec::from(&did_bytes[2..]))?;
                self.key_cache.insert(did.clone(), Arc::new(key));

                self.key_cache
                    .get(&did)
                    .ok_or_else(|| anyhow!("Couldn't find cached key"))
                    .map(|key| key.clone())
            }
            None => Err(anyhow!("Unrecognized magic bytes: {:?}", magic_bytes)),
        }
    }
}
