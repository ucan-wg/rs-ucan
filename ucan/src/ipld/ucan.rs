use cid::Cid;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{convert::TryFrom, str::FromStr};

use crate::crypto::JwtSignatureAlgorithm;
use crate::serde::Base64Encode;
use crate::{capability::CapabilityIpld, ucan::Ucan};

use crate::ipld::{Principle, Signature};
use crate::ucan::UcanPayload;
use crate::ucan::{UcanHeader, UCAN_VERSION};

#[derive(Serialize, Deserialize)]
pub struct UcanIpld {
    pub v: String,

    pub iss: Principle,
    pub aud: Principle,
    pub s: Signature,

    pub att: Vec<CapabilityIpld>,
    pub prf: Vec<Cid>,
    pub exp: u64,
    pub fct: Vec<Value>,

    pub nnc: Option<String>,
    pub nbf: Option<u64>,
}

impl TryFrom<&Ucan> for UcanIpld {
    type Error = anyhow::Error;

    fn try_from(ucan: &Ucan) -> Result<Self, Self::Error> {
        let mut prf = Vec::new();
        for cid_string in ucan.proofs() {
            prf.push(Cid::try_from(cid_string.as_str())?);
        }

        Ok(UcanIpld {
            v: ucan.version().to_string(),
            iss: Principle::from_str(ucan.issuer())?,
            aud: Principle::from_str(ucan.audience())?,
            s: Signature::try_from((
                JwtSignatureAlgorithm::from_str(ucan.algorithm())?,
                ucan.signature(),
            ))?,
            att: ucan.attenuation().clone(),
            prf,
            exp: *ucan.expires_at(),
            fct: ucan.facts().clone(),
            nnc: ucan.nonce().as_ref().cloned(),
            nbf: *ucan.not_before(),
        })
    }
}

impl TryFrom<&UcanIpld> for Ucan {
    type Error = anyhow::Error;

    fn try_from(value: &UcanIpld) -> Result<Self, Self::Error> {
        let (algorithm, signature) = value.s.decode()?;

        let header = UcanHeader {
            alg: algorithm.to_string(),
            typ: "JWT".into(),
            ucv: UCAN_VERSION.into(),
        };

        let payload = UcanPayload {
            iss: value.iss.to_string(),
            aud: value.aud.to_string(),
            exp: value.exp,
            nbf: value.nbf,
            nnc: value.nnc.clone(),
            att: value.att.clone(),
            fct: value.fct.clone(),
            prf: value.prf.iter().map(|cid| cid.to_string()).collect(),
        };

        let signed_data = format!(
            "{}.{}",
            header.jwt_base64_encode()?,
            payload.jwt_base64_encode()?
        )
        .as_bytes()
        .to_vec();

        Ok(Ucan::new(header, payload, signed_data, signature))
    }
}

#[cfg(test)]
mod tests {
    use std::convert::TryFrom;

    use serde_json::json;

    use crate::{
        tests::{
            fixtures::Identities,
            helpers::{dag_cbor_roundtrip, scaffold_ucan_builder},
        },
        Ucan,
    };

    use super::UcanIpld;

    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_test_configure!(run_in_browser);

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
    async fn it_produces_canonical_jwt_despite_json_ambiguity() {
        let identities = Identities::new().await;
        let canon_builder = scaffold_ucan_builder(&identities).await.unwrap();
        let other_builder = scaffold_ucan_builder(&identities).await.unwrap();

        let canon_jwt = canon_builder
            .with_fact(json!({
                "baz": true,
                "foo": "bar"
            }))
            .build()
            .unwrap()
            .sign()
            .await
            .unwrap()
            .encode()
            .unwrap();

        let other_jwt = other_builder
            .with_fact(json!({
                "foo": "bar",
                "baz": true
            }))
            .build()
            .unwrap()
            .sign()
            .await
            .unwrap()
            .encode()
            .unwrap();

        assert_eq!(canon_jwt, other_jwt);
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
    async fn it_stays_canonical_when_converting_between_jwt_and_ipld() {
        let identities = Identities::new().await;
        let builder = scaffold_ucan_builder(&identities).await.unwrap();

        let jwt = builder
            .with_fact(json!({
                "baz": true,
                "foo": "bar"
            }))
            .with_nonce()
            .build()
            .unwrap()
            .sign()
            .await
            .unwrap()
            .encode()
            .unwrap();

        let ucan = Ucan::try_from(jwt.as_str()).unwrap();
        let ucan_ipld = UcanIpld::try_from(&ucan).unwrap();

        let decoded_ucan_ipld = dag_cbor_roundtrip(&ucan_ipld).unwrap();

        let decoded_ucan = Ucan::try_from(&decoded_ucan_ipld).unwrap();

        let decoded_jwt = decoded_ucan.encode().unwrap();

        assert_eq!(jwt, decoded_jwt);
    }
}
