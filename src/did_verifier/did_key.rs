//! did:key method verifier

use core::fmt;
use std::{any::TypeId, collections::HashMap};

use anyhow::anyhow;
use multibase::Base;

use super::DidVerifier;

/// A closure for verifying a signature
pub type SignatureVerifier = dyn Fn(&[u8], &[u8], &[u8]) -> Result<(), anyhow::Error>;

/// did:key method verifier
#[derive(Default)]
pub struct DidKeyVerifier {
    /// map from type id of signature to verifier function
    verifier_map: HashMap<TypeId, Box<SignatureVerifier>>,
}

impl DidKeyVerifier {
    /// set verifier function for type `T`
    pub fn set<T, F>(&mut self, f: F) -> &mut Self
    where
        T: 'static,
        F: Fn(&[u8], &[u8], &[u8]) -> Result<(), anyhow::Error> + 'static,
    {
        self.verifier_map.insert(TypeId::of::<T>(), Box::new(f));
        self
    }

    /// check if verifier function for type `T` is set
    pub fn has<T>(&self) -> bool
    where
        T: 'static,
    {
        self.verifier_map.contains_key(&TypeId::of::<T>())
    }
}

impl DidVerifier for DidKeyVerifier {
    fn method(&self) -> &'static str {
        "key"
    }

    fn verify(
        &self,
        identifier: &str,
        payload: &[u8],
        signature: &[u8],
    ) -> Result<(), anyhow::Error> {
        let (base, data) = multibase::decode(identifier).map_err(|e| anyhow!(e))?;

        let Base::Base58Btc = base else {
            return Err(anyhow!("expected base58btc, got {:?}", base));
        };

        let (multicodec, public_key) =
            unsigned_varint::decode::u128(&data).map_err(|e| anyhow!(e))?;

        let multicodec_pub_key = MulticodecPubKey::try_from(multicodec)?;

        multicodec_pub_key.validate_pub_key_len(public_key)?;

        let verifier = match multicodec_pub_key {
            MulticodecPubKey::Secp256k1Compressed => self
                .verifier_map
                .get(&TypeId::of::<ecdsa::Signature<k256::Secp256k1>>()),
            MulticodecPubKey::X25519 => return Err(anyhow!("x25519 not supported for signing")),
            MulticodecPubKey::Ed25519 => self.verifier_map.get(&TypeId::of::<ed25519::Signature>()),
            MulticodecPubKey::P256Compressed => self
                .verifier_map
                .get(&TypeId::of::<ecdsa::Signature<p256::NistP256>>()),
            MulticodecPubKey::P384Compressed => self
                .verifier_map
                .get(&TypeId::of::<ecdsa::Signature<p384::NistP384>>()),
            MulticodecPubKey::P521Compressed => self
                .verifier_map
                .get(&TypeId::of::<ecdsa::Signature<p521::NistP521>>()),
            MulticodecPubKey::RSAPKCS1 => self
                .verifier_map
                .get(&TypeId::of::<rsa::pkcs1v15::Signature>()),
        }
        .ok_or_else(|| anyhow!("no registered verifier for signature type"))?;

        verifier(public_key, payload, signature)
    }
}

impl fmt::Debug for DidKeyVerifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("DidKeyVerifier").finish()
    }
}

/// Multicodec public key
#[derive(Debug)]
pub enum MulticodecPubKey {
    /// secp256k1 compressed public key
    Secp256k1Compressed,
    /// x25519 public key
    X25519,
    /// ed25519 public key
    Ed25519,
    /// p256 compressed public key
    P256Compressed,
    /// p384 compressed public key
    P384Compressed,
    /// p521 compressed public key
    P521Compressed,
    /// rsa pkcs1 public key
    RSAPKCS1,
}

impl MulticodecPubKey {
    fn validate_pub_key_len(&self, pub_key: &[u8]) -> Result<(), anyhow::Error> {
        match self {
            MulticodecPubKey::Secp256k1Compressed => {
                if pub_key.len() != 33 {
                    return Err(anyhow!(
                        "expected 33 bytes for secp256k1 compressed public key, got {}",
                        pub_key.len()
                    ));
                }
            }
            MulticodecPubKey::X25519 => {
                if pub_key.len() != 32 {
                    return Err(anyhow!(
                        "expected 32 bytes for x25519 public key, got {}",
                        pub_key.len()
                    ));
                }
            }
            MulticodecPubKey::Ed25519 => {
                if pub_key.len() != 32 {
                    return Err(anyhow!(
                        "expected 32 bytes for ed25519 public key, got {}",
                        pub_key.len()
                    ));
                }
            }
            MulticodecPubKey::P256Compressed => {
                if pub_key.len() != 33 {
                    return Err(anyhow!(
                        "expected 33 bytes for p256 compressed public key, got {}",
                        pub_key.len()
                    ));
                }
            }
            MulticodecPubKey::P384Compressed => {
                if pub_key.len() != 49 {
                    return Err(anyhow!(
                        "expected 49 bytes for p384 compressed public key, got {}",
                        pub_key.len()
                    ));
                }
            }
            MulticodecPubKey::P521Compressed => {
                if pub_key.len() > 67 {
                    return Err(anyhow!(
                        "expected <= 67 bytes for p521 compressed public key, got {}",
                        pub_key.len()
                    ));
                }
            }
            MulticodecPubKey::RSAPKCS1 => match pub_key.len() {
                94 | 126 | 162 | 226 | 294 | 422 | 546 => {}
                n => {
                    return Err(anyhow!(
                            "expected 94, 126, 162, 226, 294, 422, or 546 bytes for RSA PKCS1 public key, got {}",
                            n
                        ));
                }
            },
        };

        Ok(())
    }
}

impl TryFrom<u128> for MulticodecPubKey {
    type Error = anyhow::Error;

    fn try_from(value: u128) -> Result<Self, Self::Error> {
        match value {
            0xe7 => Ok(MulticodecPubKey::Secp256k1Compressed),
            0xec => Ok(MulticodecPubKey::X25519),
            0xed => Ok(MulticodecPubKey::Ed25519),
            0x1200 => Ok(MulticodecPubKey::P256Compressed),
            0x1201 => Ok(MulticodecPubKey::P384Compressed),
            0x1202 => Ok(MulticodecPubKey::P521Compressed),
            0x1205 => Ok(MulticodecPubKey::RSAPKCS1),
            _ => Err(anyhow!("unsupported multicodec")),
        }
    }
}
