use crate::rsa::{bytes_to_rsa_key, RSA_MAGIC_BYTES};
use crate::rsa::{RsaKeyMaterial, RSA_ALGORITHM};
use anyhow::{anyhow, Result};
use async_trait::async_trait;
use js_sys::{Array, ArrayBuffer, Boolean, Date, Object, Promise, Reflect, Uint8Array};
use rsa::pkcs1::der::Encodable;
use rsa::pkcs1::DecodeRsaPublicKey;
use rsa::RsaPublicKey;
use ucan::builder::{Signable, UcanBuilder};
use ucan::crypto::{did::DidParser, KeyMaterial};
use ucan::ucan::Ucan;
use wasm_bindgen::prelude::wasm_bindgen;
use wasm_bindgen::{JsCast, JsError, JsValue};
use wasm_bindgen_futures::{future_to_promise, JsFuture};
use web_sys::{Crypto, CryptoKey, CryptoKeyPair, SubtleCrypto};

pub fn convert_spki_to_rsa_public_key(spki_bytes: &[u8]) -> Result<Vec<u8>> {
    // TODO: This is maybe a not-good, hacky solution; verifying the first
    // 24 bytes would be more wholesome
    // SEE: https://github.com/ucan-wg/ts-ucan/issues/30#issuecomment-1007333500
    Ok(Vec::from(&spki_bytes[24..]))
}

#[derive(Debug)]
pub struct WasmError(anyhow::Error);

impl From<WasmError> for JsValue {
    fn from(err: WasmError) -> JsValue {
        JsError::new(&format!("{:?}", err)).into()
    }
}

impl From<anyhow::Error> for WasmError {
    fn from(err: anyhow::Error) -> Self {
        Self(err)
    }
}

type WasmResult<T> = std::result::Result<T, WasmError>;

#[derive(Clone)]
#[wasm_bindgen]
pub struct WebCryptoRsaKeyMaterial {
    public_key: CryptoKey,
    private_key: Option<CryptoKey>,
}

#[wasm_bindgen]
impl WebCryptoRsaKeyMaterial {
    fn get_subtle_crypto() -> Result<SubtleCrypto> {
        // NOTE: Accessing either `Window` or `DedicatedWorkerGlobalScope` in
        // a context where they are not defined will cause a JS error, so we
        // do a sneaky workaround here:
        let global = js_sys::global();
        match Reflect::get(&global, &JsValue::from("crypto")) {
            Ok(value) => Ok(value.dyn_into::<Crypto>().expect("Unexpected API").subtle()),
            _ => Err(anyhow!("Could not access WebCrypto API")),
        }
    }

    fn private_key(&self) -> Result<&CryptoKey> {
        match &self.private_key {
            Some(key) => Ok(key),
            None => Err(anyhow!("No private key configured")),
        }
    }

    #[wasm_bindgen]
    pub async fn generate(key_size: Option<u32>) -> WasmResult<WebCryptoRsaKeyMaterial> {
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

        Ok(WebCryptoRsaKeyMaterial {
            public_key,
            private_key: Some(private_key),
        })
    }

    #[wasm_bindgen(js_name = "getDid")]
    pub fn wasm_get_did(&self) -> WasmResult<Promise> {
        let me = self.clone();

        Ok(future_to_promise(async move {
            let did = me.get_did().await.map_err(|err| WasmError::from(err))?;
            Ok(JsValue::from_str(&did))
        }))
    }

    #[wasm_bindgen(js_name = "sign")]
    pub fn wasm_sign(&self, payload: &[u8]) -> WasmResult<Promise> {
        let me = self.clone();
        let payload = payload.to_vec();

        Ok(future_to_promise(async move {
            let res = me
                .sign(&payload)
                .await
                .map_err(|err| WasmError::from(err))?;
            Ok(JsValue::from(Uint8Array::from(res.as_slice())))
        }))
    }

    #[wasm_bindgen(js_name = "verify")]
    pub fn wasm_verify(&self, payload: &[u8], signature: &[u8]) -> WasmResult<Promise> {
        let me = self.clone();
        let payload = payload.to_vec();
        let signature = signature.to_vec();

        Ok(future_to_promise(async move {
            me.verify(&payload, &signature)
                .await
                .map_err(|err| WasmError::from(err))?;
            Ok(JsValue::UNDEFINED)
        }))
    }

    #[wasm_bindgen(js_name = "jwtAlgorithm")]
    pub fn wasm_jwt_algorithm(&self) -> String {
        self.get_jwt_algorithm_name()
    }
}

#[async_trait(?Send)]
impl KeyMaterial for WebCryptoRsaKeyMaterial {
    fn get_jwt_algorithm_name(&self) -> String {
        RSA_ALGORITHM.into()
    }

    async fn get_did(&self) -> Result<String> {
        let public_key = &self.public_key;
        let subtle_crypto = Self::get_subtle_crypto()?;

        let public_key_bytes = Uint8Array::new(
            &JsFuture::from(
                subtle_crypto
                    .export_key("spki", public_key)
                    .expect("Could not access key extraction API"),
            )
            .await
            .expect("Failed to extract public key bytes")
            .dyn_into::<ArrayBuffer>()
            .expect("Bytes were not an ArrayBuffer"),
        );

        let public_key_bytes = public_key_bytes.to_vec();
        let public_key_bytes = convert_spki_to_rsa_public_key(public_key_bytes.as_slice())?;

        let public_key = rsa::pkcs1::RsaPublicKey::try_from(public_key_bytes.as_slice())?;
        let public_key = RsaPublicKey::from_pkcs1_der(&public_key.to_vec()?)?;

        Ok(RsaKeyMaterial(public_key, None).get_did().await?)
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
        let key = &self.public_key;
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

        let signature = unsafe { Uint8Array::view(signature) };
        let data = unsafe { Uint8Array::view(payload) };

        let valid = JsFuture::from(
            subtle_crypto
                .verify_with_object_and_buffer_source_and_buffer_source(
                    &algorithm, key, &signature, &data,
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

#[wasm_bindgen]
pub struct WasmUcan {
    inner: Ucan,
}

#[wasm_bindgen]
impl WasmUcan {
    #[wasm_bindgen(js_name = "fromToken")]
    pub fn from_token(token: &str) -> WasmResult<WasmUcan> {
        let ucan = Ucan::try_from_token_string(token).map_err(|err| WasmError::from(err))?;
        Ok(WasmUcan { inner: ucan })
    }

    #[wasm_bindgen]
    pub fn validate(&self) -> WasmResult<Promise> {
        let ucan = self.inner.clone();

        Ok(future_to_promise(async move {
            let mut did_parser = DidParser::new(&[(RSA_MAGIC_BYTES, bytes_to_rsa_key)]);
            ucan.validate(&mut did_parser)
                .await
                .map_err(|err| WasmError::from(err))?;
            Ok(JsValue::TRUE)
        }))
    }

    #[wasm_bindgen(js_name = "checkSignature")]
    pub fn check_signature(&self) -> WasmResult<Promise> {
        let ucan = self.inner.clone();

        Ok(future_to_promise(async move {
            let mut did_parser = DidParser::new(&[(RSA_MAGIC_BYTES, bytes_to_rsa_key)]);
            ucan.check_signature(&mut did_parser)
                .await
                .map_err(|err| WasmError::from(err))?;
            Ok(JsValue::TRUE)
        }))
    }

    #[wasm_bindgen]
    pub fn encode(&self) -> WasmResult<String> {
        self.inner.encode().map_err(|err| WasmError::from(err))
    }

    #[wasm_bindgen(js_name = "isExpired")]
    pub fn is_expired(&self) -> bool {
        self.inner.is_expired()
    }

    #[wasm_bindgen(js_name = "isTooEarly")]
    pub fn is_too_early(&self) -> bool {
        self.inner.is_too_early()
    }

    #[wasm_bindgen(js_name = "signedData")]
    pub fn signed_data(&self) -> Vec<u8> {
        self.inner.signed_data().to_vec()
    }

    #[wasm_bindgen]
    pub fn algorithm(&self) -> String {
        self.inner.algorithm().to_string()
    }

    #[wasm_bindgen]
    pub fn issuer(&self) -> String {
        self.inner.issuer().to_string()
    }

    #[wasm_bindgen]
    pub fn audience(&self) -> String {
        self.inner.audience().to_string()
    }

    #[wasm_bindgen]
    pub fn proofs(&self) -> Vec<JsValue> {
        self.inner
            .proofs()
            .into_iter()
            .map(|proof| JsValue::from_str(proof))
            .collect()
    }

    #[wasm_bindgen]
    pub fn expires_at(&self) -> Date {
        // The UCAN value is the Unix Timestamp in seconds, but
        // Date expects milliseconds since EPOCH.
        let millis: JsValue = (1000 * self.inner.expires_at()).into();
        Date::new(&millis)
    }

    #[wasm_bindgen(js_name = "notBefore")]
    pub fn not_before(&self) -> Option<Date> {
        // The UCAN value is the Unix Timestamp in seconds, but
        // Date expects milliseconds since EPOCH.
        self.inner.not_before().map(|time| {
            let millis: JsValue = (1000 * time).into();
            Date::new(&millis)
        })
    }

    #[wasm_bindgen]
    pub fn nonce(&self) -> Option<String> {
        self.inner.nonce().clone()
    }

    #[wasm_bindgen(js_name = "lifetimeBeginsBefore")]
    pub fn lifetime_begins_before(&self, other: &WasmUcan) -> bool {
        self.inner.lifetime_begins_before(&other.inner)
    }

    #[wasm_bindgen(js_name = "lifetimeEndsAfter")]
    pub fn lifetime_ends_after(&self, other: &WasmUcan) -> bool {
        self.inner.lifetime_ends_after(&other.inner)
    }

    #[wasm_bindgen(js_name = "lifetimeEncompasses")]
    pub fn lifetime_encompasses(&self, other: &WasmUcan) -> bool {
        self.inner.lifetime_encompasses(&other.inner)
    }

    #[wasm_bindgen]
    pub fn attenuation(&self) -> Vec<JsValue> {
        self.inner
            .attenuation()
            .into_iter()
            .filter_map(|att| JsValue::from_serde(&att).ok())
            .collect()
    }

    #[wasm_bindgen]
    pub fn facts(&self) -> Vec<JsValue> {
        self.inner
            .facts()
            .into_iter()
            .filter_map(|fact| JsValue::from_serde(&fact).ok())
            .collect()
    }
}

#[wasm_bindgen]
pub struct WasmSignable {
    inner: Signable<WebCryptoRsaKeyMaterial>,
}

#[wasm_bindgen]
impl WasmSignable {
    pub fn sign(&self) -> WasmResult<Promise> {
        let signable = self.inner.clone();

        Ok(future_to_promise(async move {
            let inner = signable.sign().await.map_err(|err| WasmError::from(err))?;
            let ucan = WasmUcan { inner };
            Ok(ucan.into())
        }))
    }
}

#[wasm_bindgen]
pub struct WasmUcanBuilder {
    inner: UcanBuilder<WebCryptoRsaKeyMaterial>,
}

#[wasm_bindgen]
impl WasmUcanBuilder {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Self {
        Self { inner: UcanBuilder::default() }
    }

    #[wasm_bindgen(js_name = "issuedBy")]
    pub fn issued_by(self, issuer: &WebCryptoRsaKeyMaterial) -> Self {
        Self {
            inner: self.inner.issued_by(issuer),
        }
    }

    #[wasm_bindgen(js_name = "forAudience")]
    pub fn for_audience(self, audience: &str) -> Self {
        Self {
            inner: self.inner.for_audience(audience),
        }
    }

    #[wasm_bindgen(js_name = "withLifetime")]
    pub fn with_lifetime(self, seconds: u64) -> Self {
        Self {
            inner: self.inner.with_lifetime(seconds),
        }
    }

    #[wasm_bindgen(js_name = "withExpiration")]
    pub fn with_expiration(self, timestamp: Date) -> Self {
        // We need the timestamp in seconds.
        let seconds = timestamp.get_time() as u64 / 1000;
        Self {
            inner: self.inner.with_expiration(seconds),
        }
    }

    #[wasm_bindgen(js_name = "notBefore")]
    pub fn not_before(self, timestamp: Date) -> Self {
        // We need the timestamp in seconds.
        let seconds = timestamp.get_time() as u64 / 1000;
        Self {
            inner: self.inner.not_before(seconds),
        }
    }

    #[wasm_bindgen]
    pub fn with_nonce(self) -> Self {
        Self {
            inner: self.inner.with_nonce(),
        }
    }

    #[wasm_bindgen(js_name = "witnessedBy")]
    pub fn witnessed_by(self, authority: &WasmUcan) -> Self {
        Self {
            inner: self.inner.witnessed_by(&authority.inner),
        }
    }

    #[wasm_bindgen(js_name = "delegatingFrom")]
    pub fn delegating_from(self, authority: &WasmUcan) -> Self {
        Self {
            inner: self.inner.delegating_from(&authority.inner),
        }
    }

    #[wasm_bindgen]
    pub fn build(self) -> WasmResult<WasmSignable> {
        self.inner
            .build()
            .map(|inner| WasmSignable { inner })
            .map_err(|err| WasmError::from(err))
    }
}

#[cfg(test)]
mod tests {
    use wasm_bindgen_test::*;

    wasm_bindgen_test_configure!(run_in_browser);

    use super::WebCryptoRsaKeyMaterial;
    use crate::rsa::{bytes_to_rsa_key, RSA_MAGIC_BYTES};
    use ucan::{
        builder::UcanBuilder,
        crypto::{did::DidParser, KeyMaterial},
        ucan::Ucan,
    };

    #[wasm_bindgen_test]
    async fn it_can_sign_and_verify_data() {
        let key_material = WebCryptoRsaKeyMaterial::generate(None).await.unwrap();
        let data = &[0xdeu8, 0xad, 0xbe, 0xef];
        let signature = key_material.sign(data).await.unwrap();

        key_material.verify(data, signature.as_ref()).await.unwrap();
    }

    #[wasm_bindgen_test]
    async fn it_produces_a_legible_rsa_did() {
        let key_material = WebCryptoRsaKeyMaterial::generate(None).await.unwrap();
        let did = key_material.get_did().await.unwrap();
        let mut did_parser = DidParser::new(&[(RSA_MAGIC_BYTES, bytes_to_rsa_key)]);

        did_parser.parse(&did).unwrap();
    }

    #[wasm_bindgen_test]
    async fn it_signs_ucans_that_can_be_verified_elsewhere() {
        let key_material = WebCryptoRsaKeyMaterial::generate(None).await.unwrap();

        let token = UcanBuilder::default()
            .issued_by(&key_material)
            .for_audience(key_material.get_did().await.unwrap().as_str())
            .with_lifetime(300)
            .build()
            .unwrap()
            .sign()
            .await
            .unwrap()
            .encode()
            .unwrap();

        let mut did_parser = DidParser::new(&[(RSA_MAGIC_BYTES, bytes_to_rsa_key)]);
        let ucan = Ucan::try_from_token_string(token.as_str()).unwrap();

        ucan.check_signature(&mut did_parser).await.unwrap();
    }
}
