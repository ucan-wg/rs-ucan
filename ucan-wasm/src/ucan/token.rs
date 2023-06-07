use crate::ucan::JsResult;
use ::ucan::{capability::CapabilityIpld, Ucan as RsUcan};
use base64::Engine;
use js_sys::Error;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_wasm_bindgen::Serializer;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(typescript_custom_section)]
const UCAN: &'static str = r#"
interface Ucan {
    header: {
        alg: string,
        typ: string,
        ucv: string
    },
    payload: {
        iss: string,
        aud: string,
        exp: number,
        nbf?: number,
        nnc?: string,
        att: unknown[],
        fct?: Record<string,unknown>[],
        prf?: string[]
    }
    signature: string
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Ucan")]
    pub type Ucan;
}

#[wasm_bindgen]
#[derive(Debug, Serialize, Deserialize)]
pub struct ResolvedUcan {
    header: Header,
    payload: Payload,
    signature: String,
}

#[wasm_bindgen]
#[derive(Debug, Serialize, Deserialize)]
pub struct Header {
    alg: String,
    typ: String,
    ucv: String,
}

#[wasm_bindgen]
#[derive(Debug, Serialize, Deserialize)]
pub struct Payload {
    iss: String,
    aud: String,
    exp: u64,
    nbf: Option<u64>,
    nnc: Option<String>,
    att: Vec<CapabilityIpld>,
    fct: Option<Vec<Value>>,
    prf: Option<Vec<String>>,
}

/// Decode a UCAN
#[wasm_bindgen(js_name = "decode")]
pub async fn decode(token: String) -> JsResult<Ucan> {
    let ucan = RsUcan::try_from(token).map_err(|e| Error::new(&format!("{e}")))?;

    let header = Header {
        alg: ucan.algorithm().into(),
        typ: "JWT".into(),
        ucv: ucan.version().into(),
    };

    let payload = Payload {
        iss: ucan.issuer().into(),
        aud: ucan.audience().into(),
        exp: *ucan.expires_at(),
        nbf: *ucan.not_before(),
        nnc: ucan.nonce().clone(),
        att: ucan.attenuation().to_vec(),
        fct: ucan.facts().clone(),
        prf: ucan.proofs().clone(),
    };

    let signature = base64::engine::general_purpose::URL_SAFE_NO_PAD.encode(ucan.signature());

    let resolved = ResolvedUcan {
        header,
        payload,
        signature,
    };

    let serializer = Serializer::new().serialize_maps_as_objects(true);
    let value = resolved.serialize(&serializer).unwrap();

    Ok(Ucan { obj: value })
}
