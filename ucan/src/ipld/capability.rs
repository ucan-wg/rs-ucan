use crate::serde::ser_to_lower_case;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct CapabilityIpld {
    pub with: String,
    #[serde(serialize_with = "ser_to_lower_case")]
    pub can: String,
    pub nb: Option<Value>,
}

#[cfg(test)]
mod tests {
    use super::CapabilityIpld;
    use crate::tests::helpers::dag_cbor_roundtrip;
    use serde_json::json;

    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_test_configure!(run_in_browser);

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[cfg_attr(not(target_arch = "wasm32"), test)]
    fn it_lower_cases_capability_can_field() {
        let capability = dag_cbor_roundtrip(&CapabilityIpld {
            with: "foo:bar".into(),
            can: "Baz".into(),
            nb: None,
        })
        .unwrap();

        assert_eq!(capability.can, "baz");
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[cfg_attr(not(target_arch = "wasm32"), test)]
    fn it_round_trips_a_capability_with_nb() {
        let capability = dag_cbor_roundtrip(&CapabilityIpld {
            with: "foo:bar".into(),
            can: "Baz".into(),
            nb: Some(json!({ "foo": "bar" })),
        })
        .unwrap();

        assert_eq!(capability.nb, Some(json!({ "foo": "bar" })));
    }
}
