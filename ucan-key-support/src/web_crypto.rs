use crate::rsa::RSA_ALGORITHM;
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use js_sys::{Boolean, Object, Reflect, Uint8Array};
use ucan::crypto::KeyMaterial;
use wasm_bindgen::{JsCast, JsValue};
use wasm_bindgen_futures::JsFuture;
use web_sys::{CryptoKey, DedicatedWorkerGlobalScope, SubtleCrypto};

pub struct WebCryptoRsaKeyMaterial<'a>(pub &'a CryptoKey, pub Option<&'a CryptoKey>);

impl WebCryptoRsaKeyMaterial<'_> {
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
        match self.1 {
            Some(key) => Ok(key),
            None => Err(anyhow!("No private key configured")),
        }
    }
}

#[async_trait(?Send)]
impl<'a> KeyMaterial for WebCryptoRsaKeyMaterial<'a> {
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
        let key = self.0;
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

    use crate::fixtures::web::generate_web_crypto_rsa_key_pair;

    use super::WebCryptoRsaKeyMaterial;
    use ucan::crypto::KeyMaterial;

    #[wasm_bindgen_test]
    async fn it_can_sign_and_verify_data() {
        let (public_key, private_key) = generate_web_crypto_rsa_key_pair().await.unwrap();
        let key_material = WebCryptoRsaKeyMaterial(&public_key, Some(&private_key));
        let data = &[0xdeu8, 0xad, 0xbe, 0xef];
        let signature = key_material.sign(data).await.unwrap();

        key_material.verify(data, signature.as_ref()).await.unwrap();
    }
}
