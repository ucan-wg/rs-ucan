use anyhow::{anyhow, bail};
use async_signature::AsyncSigner;
use async_trait::async_trait;
use js_sys::{Date, Error, Reflect, Uint8Array};
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use wasm_bindgen::prelude::*;
use wasm_bindgen_futures::JsFuture;
use web_sys::{Crypto, CryptoKey, CryptoKeyPair, SubtleCrypto};

use crate::{builder::UcanBuilder, capability::DefaultCapabilityParser, crypto::SignerDid};

/// Convenience alias around `Result<T, js_sys::Error>`
pub type JsResult<T> = Result<T, js_sys::Error>;

/// A UCAN whose facts are a JSON value
#[wasm_bindgen]
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Ucan {
    ucan: crate::ucan::Ucan<serde_json::Value, DefaultCapabilityParser>,
}

struct JsSigner<K> {
    key_pair: CryptoKeyPair,
    _marker: std::marker::PhantomData<fn() -> K>,
}

impl<K> JsSigner<K> {
    fn subtle_crypto() -> Result<SubtleCrypto, anyhow::Error> {
        let global = js_sys::global();

        match Reflect::get(&global, &JsValue::from_str("crypto")) {
            Ok(value) => {
                let crypto = value
                    .dyn_into::<Crypto>()
                    .map_err(|_| anyhow!("Failed to cast value to Crypto"))?
                    .subtle();

                Ok(crypto)
            }
            Err(_) => bail!("Failed to get crypto from global object"),
        }
    }

    fn new(key_pair: CryptoKeyPair) -> Self {
        Self {
            key_pair,
            _marker: std::marker::PhantomData,
        }
    }

    fn signing_key(&self) -> Result<CryptoKey, anyhow::Error> {
        match Reflect::get(&self.key_pair, &JsValue::from_str("privateKey")) {
            Ok(key) => key
                .dyn_into::<CryptoKey>()
                .map_err(|_| anyhow!("Failed to cast value to CryptoKey")),
            Err(_) => bail!("Failed to get privateKey from CryptoKeyPair"),
        }
    }

    fn verifying_key(&self) -> Result<CryptoKey, anyhow::Error> {
        match Reflect::get(&self.key_pair, &JsValue::from_str("publicKey")) {
            Ok(key) => key
                .dyn_into::<CryptoKey>()
                .map_err(|_| anyhow!("Failed to cast value to CryptoKey")),
            Err(_) => bail!("Failed to get publicKey from CryptoKeyPair"),
        }
    }
}

impl<K> SignerDid for JsSigner<K> {
    fn did(&self) -> Result<String, anyhow::Error> {
        Ok("test".to_string())
    }
}

#[async_trait(?Send)]
impl AsyncSigner<rsa::pkcs1v15::Signature> for JsSigner<rsa::pkcs1v15::Signature> {
    async fn sign_async(
        &self,
        msg: &[u8],
    ) -> Result<rsa::pkcs1v15::Signature, async_signature::Error> {
        let subtle = Self::subtle_crypto().map_err(|e| async_signature::Error::from_source(e))?;

        // This can be done without copying using the unsafe `Uint8Array::view` method,
        // but I've opted to stick to safe APIs for now, until we benchmark signing.
        let data = Uint8Array::from(msg).buffer();

        let key = self
            .signing_key()
            .map_err(|e| async_signature::Error::from_source(e))?;

        let promise = subtle
            .sign_with_str_and_buffer_source("RSASSA-PKCS1-v1_5", &key, &data)
            .map_err(|_| async_signature::Error::new())?;

        let result = JsFuture::from(promise)
            .await
            .map_err(|_| async_signature::Error::new())?;

        let signature =
            rsa::pkcs1v15::Signature::try_from(Uint8Array::new(&result).to_vec().as_slice())
                .map_err(|_| async_signature::Error::new())?;

        Ok(signature)
    }
}

#[wasm_bindgen]
impl Ucan {
    /// Returns a boolean indicating whether the given UCAN is expired at the given date
    #[wasm_bindgen(js_name = "isExpired")]
    pub fn is_expired(&self, at_time: &Date) -> bool {
        let at_time = f64::floor(at_time.get_time() / 1000.) as u64;

        self.ucan.is_expired(at_time)
    }

    /// Returns true if the UCAN is not yet valid at the given date
    #[wasm_bindgen(js_name = "isTooEarly")]
    pub fn is_too_early(&self, at_time: &Date) -> bool {
        let at_time = f64::floor(at_time.get_time() / 1000.) as u64;

        self.ucan.is_too_early(at_time)
    }

    /// Returns the UCAN's signature as a `Uint8Array`
    #[wasm_bindgen(getter)]
    pub fn signature(&self) -> Vec<u8> {
        self.ucan.signature().to_vec()
    }

    /// Returns the `typ` field of the UCAN's JWT header
    #[wasm_bindgen(getter)]
    pub fn typ(&self) -> String {
        self.ucan.typ().to_string()
    }

    /// Returns the `alg` field of the UCAN's JWT header
    #[wasm_bindgen(getter)]
    pub fn algorithm(&self) -> String {
        self.ucan.algorithm().to_string()
    }

    /// Returns the `iss` field of the UCAN's JWT payload
    #[wasm_bindgen(getter)]
    pub fn issuer(&self) -> String {
        self.ucan.issuer().to_string()
    }

    /// Returns the `aud` field of the UCAN's JWT payload
    #[wasm_bindgen(getter)]
    pub fn audience(&self) -> String {
        self.ucan.audience().to_string()
    }

    /// Returns the `exp` field of the UCAN's JWT payload
    #[wasm_bindgen(getter, js_name = "expiresAt")]
    pub fn expires_at(&self) -> Option<Date> {
        self.ucan
            .expires_at()
            .map(|expires_at| Date::new(&JsValue::from_f64((expires_at as f64) * 1000.)))
    }

    /// Returns the `nbf` field of the UCAN's JWT payload
    #[wasm_bindgen(getter, js_name = "notBefore")]
    pub fn not_before(&self) -> Option<Date> {
        self.ucan
            .not_before()
            .map(|not_before| Date::new(&JsValue::from_f64((not_before as f64) * 1000.)))
    }

    /// Returns the `nnc` field of the UCAN's JWT payload
    #[wasm_bindgen(getter)]
    pub fn nonce(&self) -> Option<String> {
        self.ucan.nonce().map(String::to_string)
    }

    /// Returns the `fct` field of the UCAN's JWT payload
    #[wasm_bindgen(getter)]
    pub fn facts(&self) -> JsResult<JsValue> {
        self.ucan
            .facts()
            .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
            .map_err(|e| Error::new(&format!("Failed to serialize facts: {}", e)))
    }

    /// Returns the `vsn` field of the UCAN's JWT payload
    #[wasm_bindgen(getter)]
    pub fn version(&self) -> String {
        self.ucan.version().to_string()
    }

    /// Returns the CID of the UCAN
    #[wasm_bindgen]
    pub fn cid(&self) -> JsResult<String> {
        match self.ucan.to_cid(None) {
            Ok(cid) => Ok(cid.to_string()),
            Err(e) => Err(Error::new(&format!("Failed to convert to CID: {}", e))),
        }
    }
}

/// Decode a UCAN
#[wasm_bindgen]
pub async fn decode(token: String) -> JsResult<Ucan> {
    let ucan =
        crate::ucan::Ucan::from_str(&token).map_err(|e| Error::new(e.to_string().as_ref()))?;

    Ok(Ucan { ucan })
}

/// Options for building a UCAN
#[derive(Debug, Deserialize)]
pub struct BuildOptions {
    /// The lifetime of the UCAN in seconds
    #[serde(rename = "lifetimeInSeconds")]
    pub lifetime_in_seconds: Option<u64>,
    /// The expiration time of the UCAN in seconds since epoch
    pub expiration: Option<u64>,
    /// The time before which the UCAN is not valid in seconds since epoch
    #[serde(rename = "notBefore")]
    pub not_before: Option<u64>,
    /// The facts included in the UCAN
    pub facts: Option<serde_json::Value>,
    /// The proof CIDs referenced by the UCAN
    pub proofs: Option<Vec<String>>,
    /// The nonce of the UCAN
    pub nonce: Option<String>,
    // TODO: capabilities
}

/// Build a UCAN
#[wasm_bindgen]
pub async fn build(issuer: CryptoKeyPair, audience: &str, options: JsValue) -> JsResult<Ucan> {
    let options: BuildOptions =
        serde_wasm_bindgen::from_value(options).map_err(|e| Error::new(e.to_string().as_ref()))?;

    let builder =
        UcanBuilder::<serde_json::Value, DefaultCapabilityParser>::default().for_audience(audience);

    let builder = match options.lifetime_in_seconds {
        Some(lifetime_in_seconds) => builder.with_lifetime(lifetime_in_seconds),
        None => builder,
    };

    let builder = match options.expiration {
        Some(expiration) => builder.with_expiration(expiration),
        None => builder,
    };

    let builder = match options.not_before {
        Some(not_before) => builder.not_before(not_before),
        None => builder,
    };

    let builder = match options.facts {
        Some(facts) => builder.with_fact(facts),
        None => builder,
    };

    let builder = match options.nonce {
        Some(nonce) => builder.with_nonce(nonce),
        None => builder,
    };

    // TODO: proofs (need store)

    let signer = JsSigner::<rsa::pkcs1v15::Signature>::new(issuer);

    let ucan = builder
        .sign_async(&signer)
        .await
        .map_err(|e| Error::new(e.to_string().as_ref()))?;

    Ok(Ucan { ucan })
}

/// Panic hook lets us get better error messages if our Rust code ever panics.
///
/// For more details see
/// <https://github.com/rustwasm/console_error_panic_hook#readme>
#[wasm_bindgen(js_name = "setPanicHook")]
pub fn set_panic_hook() {
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
extern "C" {
    // For alerting
    pub(crate) fn alert(s: &str);
    // For logging in the console.
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

/// Return a representation of an object owned by JS.
#[macro_export]
macro_rules! value {
    ($value:expr) => {
        wasm_bindgen::JsValue::from($value)
    };
}

/// Calls the wasm_bindgen console.log.
#[macro_export]
macro_rules! console_log {
    ($($t:tt)*) => ($crate::log(&format_args!($($t)*).to_string()))
}
