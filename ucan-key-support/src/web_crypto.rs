use crate::rsa::RSA_ALGORITHM;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use js_sys::{Array, Boolean, Object, Reflect, Uint8Array};
use ucan::crypto::KeyMaterial;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{CryptoKey, CryptoKeyPair, DedicatedWorkerGlobalScope, SubtleCrypto};

pub struct WebCryptoRsaKeyMaterial(pub CryptoKey, pub Option<CryptoKey>);

impl WebCryptoRsaKeyMaterial {
    fn get_subtle_crypto() -> Result<SubtleCrypto> {
        match web_sys::window() {
            Some(window) => Ok(window
                .crypto()
                .map_err(|error| anyhow!("{:?}", error))?
                .subtle()),
            None => match js_sys::global().dyn_into::<DedicatedWorkerGlobalScope>() {
                Ok(global) => Ok(global
                    .crypto()
                    .map_err(|error| anyhow!("{:?}", error))?
                    .subtle()),
                Err(error) => Err(anyhow!("{:?}", error)),
            },
        }
    }

    fn private_key(&self) -> Result<&CryptoKey> {
        match &self.1 {
            Some(key) => Ok(key),
            None => Err(anyhow!("No private key configured")),
        }
    }

    async fn generate(key_size: Option<u32>) -> Result<WebCryptoRsaKeyMaterial> {
        let subtle_crypto = Self::get_subtle_crypto()?;
        let algorithm = Object::new();

        Reflect::set(
            &algorithm,
            &JsValue::from("name"),
            &JsValue::from(RSA_ALGORITHM),
        )
        .map_err(|error| anyhow!("{:?}", error))?;

        Reflect::set(
            &algorithm,
            &JsValue::from("modulusLength"),
            &JsValue::from(key_size.unwrap_or(2048)),
        )
        .map_err(|error| anyhow!("{:?}", error))?;

        let public_exponent = Uint8Array::new(&JsValue::from(3u8));
        public_exponent.copy_from(&[0x01u8, 0x00, 0x01]);

        Reflect::set(
            &algorithm,
            &JsValue::from("publicExponent"),
            &JsValue::from(public_exponent),
        )
        .map_err(|error| anyhow!("{:?}", error))?;

        let hash = Object::new();

        Reflect::set(&hash, &JsValue::from("name"), &JsValue::from("SHA-256"))
            .map_err(|error| anyhow!("{:?}", error))?;

        Reflect::set(&algorithm, &JsValue::from("hash"), &JsValue::from(hash))
            .map_err(|error| anyhow!("{:?}", error))?;

        let uses = Array::new();

        uses.push(&JsValue::from("sign"));
        uses.push(&JsValue::from("verify"));

        let crypto_key_pair_generates = subtle_crypto
            .generate_key_with_object(&algorithm, false, &uses)
            .map_err(|error| anyhow!("{:?}", error))?;
        let crypto_key_pair = CryptoKeyPair::from(
            JsFuture::from(crypto_key_pair_generates)
                .await
                .map_err(|error| anyhow!("{:?}", error))?,
        );

        let public_key = CryptoKey::from(
            Reflect::get(&crypto_key_pair, &JsValue::from("publicKey"))
                .map_err(|error| anyhow!("{:?}", error))?,
        );
        let private_key = CryptoKey::from(
            Reflect::get(&crypto_key_pair, &JsValue::from("privateKey"))
                .map_err(|error| anyhow!("{:?}", error))?,
        );

        Ok(WebCryptoRsaKeyMaterial(public_key, Some(private_key)))
    }
}

#[async_trait(?Send)]
impl KeyMaterial for WebCryptoRsaKeyMaterial {
    fn get_jwt_algorithm_name(&self) -> String {
        RSA_ALGORITHM.into()
    }

    fn get_did(&self) -> String {
        todo!()
    }

    async fn sign(&self, payload: &[u8]) -> Result<Vec<u8>> {
        let key = self.private_key()?;
        let subtle_crypto = Self::get_subtle_crypto()?;
        let algorithm = Object::new();

        Reflect::set(
            &algorithm,
            &JsValue::from("name"),
            &JsValue::from(RSA_ALGORITHM),
        )
        .map_err(|error| anyhow!("{:?}", error))?;

        Reflect::set(
            &algorithm,
            &JsValue::from("saltLength"),
            &JsValue::from(128u8),
        )
        .map_err(|error| anyhow!("{:?}", error))?;

        let data = unsafe { Uint8Array::view(payload) };

        let result = Uint8Array::new(
            &JsFuture::from(
                subtle_crypto
                    .sign_with_object_and_buffer_source(&algorithm, key, &data)
                    .map_err(|error| anyhow!("{:?}", error))?,
            )
            .await
            .map_err(|error| anyhow!("{:?}", error))?,
        );

        Ok(result.to_vec())
    }

    async fn verify(&self, payload: &[u8], signature: &[u8]) -> Result<()> {
        let key = &self.0;
        let subtle_crypto = Self::get_subtle_crypto()?;
        let algorithm = Object::new();

        Reflect::set(
            &algorithm,
            &JsValue::from("name"),
            &JsValue::from(RSA_ALGORITHM),
        )
        .map_err(|error| anyhow!("{:?}", error))?;
        Reflect::set(
            &algorithm,
            &JsValue::from("saltLength"),
            &JsValue::from(128u8),
        )
        .map_err(|error| anyhow!("{:?}", error))?;

        let signature = unsafe { Uint8Array::view(signature.as_ref()) };
        let data = unsafe { Uint8Array::view(payload.as_ref()) };

        let valid = JsFuture::from(
            subtle_crypto
                .verify_with_object_and_buffer_source_and_buffer_source(
                    &algorithm, &key, &signature, &data,
                )
                .map_err(|error| anyhow!("{:?}", error))?,
        )
        .await
        .map_err(|error| anyhow!("{:?}", error))?
        .dyn_into::<Boolean>()
        .map_err(|error| anyhow!("{:?}", error))?;

        match valid.is_truthy() {
            true => Ok(()),
            false => Err(anyhow!("Could not verify signature")),
        }
    }
}

#[cfg(test)]
mod tests {
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    use super::WebCryptoRsaKeyMaterial;
    use ucan::crypto::KeyMaterial;

    #[wasm_bindgen_test]
    async fn it_can_sign_and_verify_data() {
        let key_material = WebCryptoRsaKeyMaterial::generate(None).await.unwrap();
        let data = &[0xdeu8, 0xad, 0xbe, 0xef];
        let signature = key_material.sign(data).await.unwrap();

        key_material.verify(data, signature.as_ref()).await.unwrap();
    }
}
