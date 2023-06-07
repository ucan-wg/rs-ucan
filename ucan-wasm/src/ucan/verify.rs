use crate::ucan::JsResult;
use ::ucan::{
    crypto::did::{
        DidParser, KeyConstructorSlice, ED25519_MAGIC_BYTES, P256_MAGIC_BYTES, RSA_MAGIC_BYTES,
    },
    time::now,
    Ucan,
};
use ::ucan_key_support::{
    ed25519::bytes_to_ed25519_key, p256::bytes_to_p256_key, rsa::bytes_to_rsa_key,
};
use js_sys::Error;
use wasm_bindgen::prelude::wasm_bindgen;

const SUPPORTED_KEYS: &KeyConstructorSlice = &[
    (ED25519_MAGIC_BYTES, bytes_to_ed25519_key),
    (RSA_MAGIC_BYTES, bytes_to_rsa_key),
    (P256_MAGIC_BYTES, bytes_to_p256_key),
];

/// Validate the UCAN's signature and timestamps
#[wasm_bindgen(js_name = "validate")]
pub async fn validate(token: String) -> JsResult<()> {
    let mut did_parser = DidParser::new(SUPPORTED_KEYS);
    let ucan = Ucan::try_from(token).map_err(|e| Error::new(&format!("{e}")))?;
    let now = now();

    Ucan::validate(&ucan, Some(now), &mut did_parser)
        .await
        .map_err(|e| Error::new(&format!("{e}")))?;

    Ok(())
}

/// Validate that the signed data was signed by the stated issuer
#[wasm_bindgen(js_name = "checkSignature")]
pub async fn check_signature(token: String) -> JsResult<()> {
    let mut did_parser = DidParser::new(SUPPORTED_KEYS);
    let ucan = Ucan::try_from(token).map_err(|e| Error::new(&format!("{e}")))?;

    ucan.check_signature(&mut did_parser)
        .await
        .map_err(|e| Error::new(&format!("{e}")))?;

    Ok(())
}

/// Returns true if the UCAN has past its expiration date
#[wasm_bindgen(js_name = "isExpired")]
pub fn is_expired(token: String) -> JsResult<bool> {
    let ucan = Ucan::try_from(token).map_err(|e| Error::new(&format!("{e}")))?;
    let now = now();

    Ok(Ucan::is_expired(&ucan, Some(now)))
}

/// Returns true if the not-before ("nbf") time is still in the future
#[wasm_bindgen(js_name = "isTooEarly")]
pub fn is_too_early(token: String) -> JsResult<bool> {
    let ucan = Ucan::try_from(token).map_err(|e| Error::new(&format!("{e}")))?;

    Ok(Ucan::is_too_early(&ucan))
}
