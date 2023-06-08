use crate::ucan::JsResult;
use ::ucan::Ucan;
use cid::multihash::Code;
use js_sys::Error;
use wasm_bindgen::prelude::wasm_bindgen;

/// Compute CID for a UCAN
///
/// Hashers include SHA2-256, SHA2-512, SHA3-224
/// SHA3-256, SHA3-384, SHA3-512, Keccak-224, Keccak-256, Keccak-384
/// Keccak-512, BLAKE2b-256, BLAKE2b-512, BLAKE2s-128, and BLAKE3-256.
///
/// Defaults to BLAKE3-256 hash function.
///
#[wasm_bindgen(js_name = "toCID")]
pub async fn to_cid(token: String, hasher: Option<String>) -> JsResult<String> {
    let ucan = Ucan::try_from(token).map_err(|e| Error::new(&format!("{e}")))?;

    let hasher_code = get_hasher_code(&hasher.unwrap_or(String::from("BLAKE3-256")));
    let cid = Ucan::to_cid(&ucan, hasher_code).map_err(|e| Error::new(&format!("{e}")))?;

    Ok(cid.into())
}

fn get_hasher_code(hasher: &str) -> Code {
    match hasher {
        "SHA2-256" => Code::Sha2_256,
        "SHA2-512" => Code::Sha2_512,
        "SHA3-224" => Code::Sha3_224,
        "SHA3-256" => Code::Sha3_256,
        "SHA3-384" => Code::Sha3_384,
        "SHA3-512" => Code::Sha3_512,
        "Keccak-224" => Code::Keccak224,
        "Keccak-256" => Code::Keccak256,
        "Keccak-384" => Code::Keccak384,
        "Keccak-512" => Code::Keccak512,
        "BLAKE2b-256" => Code::Blake2b256,
        "BLAKE2b-512" => Code::Blake2b512,
        "BLAKE2s-128" => Code::Blake2s128,
        "BLAKE3-256" => Code::Blake3_256,
        _ => Code::Blake3_256,
    }
}
