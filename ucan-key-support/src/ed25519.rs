use anyhow::{anyhow, Result};
use async_trait::async_trait;

use ed25519_zebra::{
    Signature, SigningKey as Ed25519PrivateKey, VerificationKey as Ed25519PublicKey,
};

use ucan::crypto::KeyMaterial;

pub const ED25519_MAGIC_BYTES: &[u8] = &[0xed, 0x01];

pub struct Ed25519KeyMaterial<'a>(pub &'a Ed25519PublicKey, Option<&'a Ed25519PrivateKey>);

#[cfg_attr(feature = "web", async_trait(?Send))]
#[cfg_attr(not(feature = "web"), async_trait)]
impl<'a> KeyMaterial for Ed25519KeyMaterial<'a> {
    fn get_jwt_algorithm_name(&self) -> String {
        "EdDSA".into()
    }

    fn get_did(&self) -> String {
        let bytes = [ED25519_MAGIC_BYTES, self.0.as_ref()].concat();
        format!("did:key:z{}", bs58::encode(bytes).into_string())
    }

    async fn sign(&self, payload: &[u8]) -> Result<Vec<u8>> {
        match self.1 {
            Some(private_key) => {
                let signature = private_key.sign(payload);
                let bytes: [u8; 64] = signature.into();
                Ok(bytes.to_vec())
            }
            None => Err(anyhow!("No private key; cannot sign data")),
        }
    }

    async fn verify(&self, payload: &[u8], signature: &[u8]) -> Result<()> {
        let signature = Signature::try_from(signature)?;
        self.0
            .verify(&signature, payload)
            .map_err(|error| anyhow!(error))
    }
}

#[cfg(test)]
mod tests {
    use super::Ed25519KeyMaterial;
    use ed25519_zebra::SigningKey as Ed25519PrivateKey;
    use ed25519_zebra::VerificationKey as Ed25519PublicKey;
    use ucan::crypto::KeyMaterial;

    #[tokio::test]
    async fn it_can_sign_and_verify_data() {
        let rng = rand::thread_rng();
        let private_key = Ed25519PrivateKey::new(rng);
        let public_key = Ed25519PublicKey::from(&private_key);

        let signing_key = Ed25519KeyMaterial(&public_key, Some(&private_key));
        let data = &[0xdeu8, 0xad, 0xbe, 0xef];
        let signature = signing_key.sign(data).await.unwrap();

        signing_key.verify(data, &signature).await.unwrap();
    }
}
