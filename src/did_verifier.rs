//! DID verifier methods

use core::fmt;
use std::collections::HashMap;

use crate::error::Error;

#[cfg(feature = "did-key")]
pub mod did_key;

/// A map from did method to verifier
#[derive(Debug)]
pub struct DidVerifierMap {
    map: HashMap<String, Box<dyn DidVerifier>>,
}

impl Default for DidVerifierMap {
    fn default() -> Self {
        #[allow(unused_mut)]
        let mut did_verifier_map = Self {
            map: HashMap::new(),
        };

        #[cfg(feature = "did-key")]
        did_verifier_map.register(did_key::DidKeyVerifier::default());

        did_verifier_map
    }
}

impl DidVerifierMap {
    /// Register a verifier
    pub fn register<V>(&mut self, verifier: V) -> &mut Self
    where
        V: DidVerifier + 'static,
    {
        self.map
            .insert(verifier.method().to_string(), Box::new(verifier));
        self
    }

    /// Register a verifier that's already boxed
    pub fn register_box(&mut self, verifier: Box<dyn DidVerifier>) -> &mut Self {
        self.map.insert(verifier.method().to_string(), verifier);
        self
    }

    /// Verify a signature using the registered verifier for the given method
    pub fn verify(
        &self,
        method: &str,
        identifier: &str,
        payload: &[u8],
        signature: &[u8],
    ) -> Result<(), Error> {
        self.map
            .get(method)
            .ok_or_else(|| Error::VerifyingError {
                msg: format!("Unrecognized DID method, {}", method),
            })?
            .verify(identifier, payload, signature)
            .map_err(|e| Error::VerifyingError { msg: e.to_string() })
    }
}

impl FromIterator<Box<dyn DidVerifier>> for DidVerifierMap {
    fn from_iter<T: IntoIterator<Item = Box<dyn DidVerifier>>>(iter: T) -> Self {
        let mut map = Self::default();
        for verifier in iter {
            map.register_box(verifier);
        }

        map
    }
}

impl Extend<Box<dyn DidVerifier>> for DidVerifierMap {
    fn extend<T: IntoIterator<Item = Box<dyn DidVerifier>>>(&mut self, iter: T) {
        for verifier in iter {
            self.register_box(verifier);
        }
    }
}

/// A trait for implementing DID method verification
pub trait DidVerifier {
    /// The DID method for this verifier
    fn method(&self) -> &'static str;

    /// Verify a signature
    fn verify(
        &self,
        identifier: &str,
        payload: &[u8],
        signature: &[u8],
    ) -> Result<(), anyhow::Error>;
}

impl fmt::Debug for dyn DidVerifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DidVerifier")
            .field("method", &self.method())
            .finish()
    }
}
