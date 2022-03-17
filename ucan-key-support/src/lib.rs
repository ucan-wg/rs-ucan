use anyhow::{anyhow, Result};
use ring::{
    rand::SecureRandom,
    signature::{
        Ed25519KeyPair, RsaKeyPair, UnparsedPublicKey, ED25519, RSA_PKCS1_2048_8192_SHA256,
        RSA_PKCS1_SHA256,
    },
};
use ucan::crypto::SigningKey;

pub const ED25519_MAGIC_BYTES: &[u8] = &[0xed, 0x01];
pub const RSA_MAGIC_BYTES: &[u8] = &[0x85, 0x24];

pub enum KeyPair<'a, B>
where
    B: AsRef<[u8]>,
{
    Ed25519(B, Option<Ed25519KeyPair>),
    RSA(B, Option<(RsaKeyPair, &'a dyn SecureRandom)>),
}

impl<'a, B> SigningKey for KeyPair<'a, B>
where
    B: AsRef<[u8]>,
{
    fn get_jwt_algorithm_name(&self) -> String {
        match self {
            KeyPair::Ed25519(_, _) => "EdDSA",
            KeyPair::RSA(_, _) => "RSASSA-PKCS1-v1_5",
        }
        .into()
    }

    fn get_did(&self) -> String {
        let bytes = match self {
            KeyPair::Ed25519(public_key_bytes, _) => {
                [ED25519_MAGIC_BYTES, public_key_bytes.as_ref()].concat()
            }
            KeyPair::RSA(public_key_bytes, _) => {
                [RSA_MAGIC_BYTES, public_key_bytes.as_ref()].concat()
            }
        };

        format!("did:key:z{}", bs58::encode(bytes).into_string())
    }

    fn sign(&self, payload: &[u8]) -> Result<Vec<u8>> {
        match self {
            KeyPair::Ed25519(_, Some(key_pair)) => Ok(Vec::from(key_pair.sign(payload).as_ref())),
            KeyPair::RSA(_, Some((key_pair, rng))) => {
                let mut signature = vec![0u8; key_pair.public_modulus_len()];
                key_pair
                    .sign(&RSA_PKCS1_SHA256, *rng, payload, signature.as_mut_slice())
                    .map_err(|_| anyhow!("Failed to sign payload"))?;
                Ok(signature)
            }
            _ => Err(anyhow!("No known signing key pair available")),
        }
    }

    fn verify(&self, payload: &[u8], signature: &[u8]) -> Result<()> {
        match self {
            KeyPair::Ed25519(public_key_bytes, _) => {
                UnparsedPublicKey::new(&ED25519, public_key_bytes)
                    .verify(payload, signature)
                    .map_err(|_| anyhow!("Signature verification failed"))
            }
            KeyPair::RSA(public_key_bytes, _) => {
                UnparsedPublicKey::new(&RSA_PKCS1_2048_8192_SHA256, public_key_bytes)
                    .verify(payload, signature)
                    .map_err(|_| anyhow!("Signature verification failed"))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::KeyPair;
    use ucan::crypto::SigningKey;

    use ring::{
        rand::SystemRandom,
        signature::{Ed25519KeyPair, KeyPair as RingKeyPair, RsaKeyPair},
    };

    #[test]
    fn it_can_sign_and_verify_with_ed25519() {
        let rng = SystemRandom::new();
        let pkcs8 = Ed25519KeyPair::generate_pkcs8(&rng).unwrap();
        let key_pair = Ed25519KeyPair::from_pkcs8(pkcs8.as_ref()).unwrap();
        let signing_key = KeyPair::Ed25519(key_pair.public_key().clone(), Some(key_pair));

        let data = &[0xdeu8, 0xad, 0xbe, 0xef];
        let signature = signing_key.sign(data).unwrap();

        signing_key.verify(data, &signature).unwrap();
    }

    #[test]
    fn it_can_sign_and_verify_with_rsa() {
        let rng = SystemRandom::new();
        let key_pair = RsaKeyPair::from_pkcs8(include_bytes!["fixtures/rsa_key.pk8"]).unwrap();
        let signing_key = KeyPair::RSA(key_pair.public_key().clone(), Some((key_pair, &rng)));

        let data = &[0xdeu8, 0xad, 0xbe, 0xef];
        let signature = signing_key.sign(data).unwrap();

        signing_key.verify(data, &signature).unwrap();
    }
}
