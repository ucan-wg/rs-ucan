use anyhow::{anyhow, Result};
use async_trait::async_trait;

use rsa::{
    pkcs1::{FromRsaPublicKey, ToRsaPublicKey},
    Hash, PaddingScheme, PublicKey, RsaPrivateKey, RsaPublicKey,
};

use sha2::{Digest, Sha256};
use ucan::crypto::KeyMaterial;

pub const RSA_MAGIC_BYTES: [u8; 2] = [0x85, 0x24];
pub const RSA_ALGORITHM: &str = "RSASSA-PKCS1-v1_5";

pub fn bytes_to_rsa_key(bytes: Vec<u8>) -> Result<Box<dyn KeyMaterial>> {
    // NOTE: DID bytes are PKCS1, but we are using PKCS8, so do the conversion here..
    println!("Trying to parse RSA key...");
    let public_key = rsa::pkcs1::RsaPublicKey::try_from(bytes.as_slice())?;
    let public_key = RsaPublicKey::from_pkcs1_public_key(public_key)?;

    Ok(Box::new(RsaKeyMaterial(public_key, None)))
}

#[derive(Clone)]
pub struct RsaKeyMaterial(pub RsaPublicKey, pub Option<RsaPrivateKey>);

#[cfg_attr(all(target_arch="wasm32", feature = "web"), async_trait(?Send))]
#[cfg_attr(any(not(target_arch = "wasm32"), not(feature = "web")), async_trait)]
impl KeyMaterial for RsaKeyMaterial {
    fn get_jwt_algorithm_name(&self) -> String {
        RSA_ALGORITHM.into()
    }

    async fn get_did(&self) -> Result<String> {
        let bytes = match self.0.to_pkcs1_der() {
            Ok(document) => [RSA_MAGIC_BYTES.as_slice(), document.as_der()].concat(),
            Err(error) => {
                // TODO: Probably shouldn't swallow this error...
                warn!("Could not get RSA public key bytes for DID: {:?}", error);
                Vec::new()
            }
        };
        Ok(format!("did:key:z{}", bs58::encode(bytes).into_string()))
    }

    async fn sign(&self, payload: &[u8]) -> Result<Vec<u8>> {
        let mut hasher = Sha256::new();
        hasher.update(payload);
        let hashed = hasher.finalize();

        match &self.1 {
            Some(private_key) => {
                let signature = private_key.sign(
                    PaddingScheme::PKCS1v15Sign {
                        hash: Some(Hash::SHA2_256),
                    },
                    hashed.as_ref(),
                )?;
                info!("SIGNED!");
                Ok(signature)
            }
            None => Err(anyhow!("No private key; cannot sign data")),
        }
    }

    async fn verify(&self, payload: &[u8], signature: &[u8]) -> Result<()> {
        let mut hasher = Sha256::new();
        hasher.update(payload);
        let hashed = hasher.finalize();

        self.0
            .verify(
                PaddingScheme::PKCS1v15Sign {
                    hash: Some(Hash::SHA2_256),
                },
                hashed.as_ref(),
                signature,
            )
            .map_err(|error| anyhow!(error))
    }
}

#[cfg(test)]
mod tests {
    use super::bytes_to_rsa_key;
    use super::RsaKeyMaterial;
    use super::RSA_MAGIC_BYTES;

    use rsa::pkcs8::FromPrivateKey;
    use rsa::RsaPrivateKey;
    use rsa::RsaPublicKey;
    use ucan::builder::UcanBuilder;
    use ucan::crypto::did::DidParser;
    use ucan::crypto::KeyMaterial;
    use ucan::ucan::Ucan;

    #[tokio::test]
    async fn it_can_sign_and_verify_a_ucan() {
        let private_key =
            RsaPrivateKey::from_pkcs8_der(include_bytes!("./fixtures/rsa_key.pk8")).unwrap();
        let public_key = RsaPublicKey::from(&private_key);

        let key_material = RsaKeyMaterial(public_key, Some(private_key));
        let token_string = UcanBuilder::default()
            .issued_by(&key_material)
            .for_audience(key_material.get_did().await.unwrap().as_str())
            .with_lifetime(60)
            .build()
            .unwrap()
            .sign()
            .await
            .unwrap()
            .encode()
            .unwrap();

        let mut did_parser = DidParser::new(&[(RSA_MAGIC_BYTES, bytes_to_rsa_key)]);

        let ucan = Ucan::try_from_token_string(token_string.as_str()).unwrap();
        ucan.check_signature(&mut did_parser).await.unwrap();
    }
}
