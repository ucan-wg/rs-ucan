use crate::ucan::JsResult;
use ::ucan::{time::now, Ucan};

use js_sys::Error;
use wasm_bindgen::prelude::wasm_bindgen;

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
