use crate::rsa::RSA_ALGORITHM;
use anyhow::{anyhow, Result};
use js_sys::{Array, Object, Reflect, Uint8Array};
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::JsFuture;
use web_sys::{CryptoKey, CryptoKeyPair};

pub async fn generate_web_crypto_rsa_key_pair() -> Result<(CryptoKey, CryptoKey)> {
    let window =
        web_sys::window().ok_or_else(|| anyhow!("Could not get a reference to document window"))?;
    let subtle_crypto = window
        .crypto()
        .map_err(|error| anyhow!("{:?}", error))?
        .subtle();
    let algorithm_config = JsValue::from(Object::new());

    Reflect::set(
        &algorithm_config,
        &JsValue::from("name"),
        &JsValue::from(RSA_ALGORITHM),
    )
    .map_err(|error| anyhow!("{:?}", error))?;

    Reflect::set(
        &algorithm_config,
        &JsValue::from("modulusLength"),
        &JsValue::from(2048i16),
    )
    .map_err(|error| anyhow!("{:?}", error))?;

    Reflect::set(
        &algorithm_config,
        &JsValue::from("publicExponent"),
        &Uint8Array::from(vec![0x01u8, 0x00, 0x01].as_slice()),
    )
    .map_err(|error| anyhow!("{:?}", error))?;

    let hash_config = JsValue::from(Object::new());

    Reflect::set(
        &hash_config,
        &JsValue::from("name"),
        &JsValue::from("SHA-256"),
    )
    .map_err(|error| anyhow!("{:?}", error))?;

    Reflect::set(&algorithm_config, &JsValue::from("hash"), &hash_config)
        .map_err(|error| anyhow!("{:?}", error))?;

    let uses = Array::new();
    uses.push(&JsValue::from("sign"));
    uses.push(&JsValue::from("verify"));

    let crypto_key_pair_generates = subtle_crypto
        .generate_key_with_object(&Object::from(algorithm_config), false, &uses)
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

    Ok((public_key, private_key))
}
