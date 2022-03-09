use std::str::FromStr;

use anyhow::{anyhow, Result};
use did_key::{CoreSign, Fingerprint};
use did_url::DID;
pub use pkcs1::ToRsaPublicKey;
use rsa::{pkcs8::FromPublicKey, Hash, PaddingScheme};
pub use rsa::{PublicKey, RsaPrivateKey, RsaPublicKey};

use super::SigningKey;

pub const MAGIC_BYTES: &[u8] = &[0x85, 0x24];

pub struct RsaKeyPair(pub RsaPublicKey, pub Option<RsaPrivateKey>);

impl Fingerprint for RsaKeyPair {
    fn fingerprint(&self) -> String {
        match self.0.to_pkcs1_der() {
            Ok(public_key) => {
                let bytes = [MAGIC_BYTES, public_key.as_der()].concat();
                format!("z{}", bs58::encode(bytes).into_string())
            }
            Err(error) => {
                warn!(
                    "Could not serialize RSA public key with PKCS1 encoding: {}",
                    error
                );
                String::from("")
            }
        }
    }
}

impl CoreSign for RsaKeyPair {
    fn sign(&self, payload: &[u8]) -> Vec<u8> {
        // NOTE: Safe to unwrap, as the result only errors when an improper
        // padding scheme is used
        match &self.1 {
            Some(private_key) => private_key
                .sign(
                    PaddingScheme::PKCS1v15Sign {
                        hash: Some(Hash::SHA2_256),
                    },
                    payload,
                )
                .unwrap(),
            None => {
                warn!("Attempt to sign without RSA private key; signature will be empty");
                Vec::new()
            }
        }
    }

    fn verify(&self, payload: &[u8], signature: &[u8]) -> Result<(), did_key::Error> {
        match self.0.verify(
            PaddingScheme::PKCS1v15Sign {
                hash: Some(Hash::SHA2_256),
            },
            payload,
            signature,
        ) {
            Err(_) => Err(did_key::Error::SignatureError),
            _ => Ok(()),
        }
    }
}

impl SigningKey for RsaKeyPair {
    fn get_jwt_algorithm_name(&self) -> String {
        "RSASSA-PKCS1-v1_5".into()
    }

    fn try_from_did(did: String) -> Result<Self> {
        let did = DID::from_str(did.as_str())?;

        let public_key = match did.method_id().strip_prefix("z") {
            Some(id) => bs58::decode(id).into_vec()?,
            None => return Err(anyhow!("Could not decode DID as RSA public key")),
        };

        if &public_key[0..2] == MAGIC_BYTES {
            Ok(RsaKeyPair(
                RsaPublicKey::from_public_key_der(&public_key[2..]).unwrap(),
                None,
            ))
        } else {
            Err(anyhow!("Incorrect header bytes for RSA public key"))
        }
    }
}
