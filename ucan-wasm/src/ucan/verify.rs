use crate::ucan::JsResult;
use ::ucan::{time::now, Ucan};

use js_sys::Error;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(js_name = "isExpired")]
pub fn is_expired(token: String) -> JsResult<bool> {
    let ucan = Ucan::try_from(token).map_err(|e| Error::new(&format!("{e}")))?;
    let now = now();

    Ok(Ucan::is_expired(&ucan, Some(now)))
}
