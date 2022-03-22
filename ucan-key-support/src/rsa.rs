use anyhow::{anyhow, Result};
use async_trait::async_trait;

use rsa::{
    pkcs1::ToRsaPublicKey, pkcs8::FromPublicKey, PaddingScheme, PublicKey, RsaPrivateKey,
    RsaPublicKey,
};

use ucan::crypto::KeyMaterial;

pub const RSA_MAGIC_BYTES: [u8; 2] = [0x85, 0x24];
pub const RSA_ALGORITHM: &str = "RSASSA-PKCS1-v1_5";

pub fn bytes_to_rsa_key(bytes: Vec<u8>) -> Result<Box<dyn KeyMaterial>> {
    let public_key = RsaPublicKey::from_public_key_der(bytes.as_slice())?;
    Ok(Box::new(RsaKeyMaterial(public_key, None)))
}

#[derive(Clone)]
pub struct RsaKeyMaterial(pub RsaPublicKey, pub Option<RsaPrivateKey>);

#[cfg_attr(feature = "web", async_trait(?Send))]
#[cfg_attr(not(feature = "web"), async_trait)]
impl KeyMaterial for RsaKeyMaterial {
    fn get_jwt_algorithm_name(&self) -> String {
        RSA_ALGORITHM.into()
    }

    fn get_did(&self) -> String {
        let bytes = match self.0.to_pkcs1_der() {
            Ok(document) => [RSA_MAGIC_BYTES.as_slice(), document.as_der()].concat(),
            Err(error) => {
                // TODO: Probably shouldn't swallow this error...
                warn!("Could not get RSA public key bytes for DID: {:?}", error);
                Vec::new()
            }
        };
        format!("did:key:z{}", bs58::encode(bytes).into_string())
    }

    async fn sign(&self, payload: &[u8]) -> Result<Vec<u8>> {
        match &self.1 {
            Some(private_key) => {
                let signature =
                    private_key.sign(PaddingScheme::PKCS1v15Sign { hash: None }, payload)?;
                Ok(signature)
            }
            None => Err(anyhow!("No private key; cannot sign data")),
        }
    }

    async fn verify(&self, payload: &[u8], signature: &[u8]) -> Result<()> {
        self.0
            .verify(
                PaddingScheme::PKCS1v15Sign { hash: None },
                payload,
                signature,
            )
            .map_err(|error| anyhow!(error))
    }
}

#[cfg(test)]
mod tests {
    use super::RsaKeyMaterial;

    use rsa::pkcs8::FromPrivateKey;
    use rsa::RsaPrivateKey;
    use rsa::RsaPublicKey;
    use ucan::crypto::KeyMaterial;

    #[tokio::test]
    async fn it_can_sign_and_verify_data() {
        let private_key =
            RsaPrivateKey::from_pkcs8_der(include_bytes!("./fixtures/rsa_key.pk8")).unwrap();
        let public_key = RsaPublicKey::from(&private_key);

        let key_material = RsaKeyMaterial(public_key, Some(private_key));
        let data = &[0xdeu8, 0xad, 0xbe, 0xef];
        let signature = key_material.sign(data).await.unwrap();

        key_material.verify(data, &signature).await.unwrap();
    }
}
