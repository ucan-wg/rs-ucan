mod validate {
    use crate::{
        builder::UcanBuilder,
        crypto::did::DidParser,
        tests::fixtures::{Identities, SUPPORTED_KEYS},
        time::now,
        ucan::Ucan,
    };

    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_test_configure!(run_in_browser);

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
    async fn it_round_trips_with_encode() {
        let identities = Identities::new().await;
        let mut did_parser = DidParser::new(SUPPORTED_KEYS);

        let ucan = UcanBuilder::default()
            .issued_by(&identities.alice_key)
            .for_audience(identities.bob_did.as_str())
            .with_lifetime(30)
            .build()
            .unwrap()
            .sign()
            .await
            .unwrap();

        let encoded_ucan = ucan.encode().unwrap();
        let decoded_ucan = Ucan::try_from(encoded_ucan.as_str()).unwrap();

        decoded_ucan.validate(&mut did_parser).await.unwrap();
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
    async fn it_identifies_a_ucan_that_is_not_active_yet() {
        let identities = Identities::new().await;

        let ucan = UcanBuilder::default()
            .issued_by(&identities.alice_key)
            .for_audience(identities.bob_did.as_str())
            .not_before(now() + 30)
            .with_lifetime(30)
            .build()
            .unwrap()
            .sign()
            .await
            .unwrap();

        assert!(ucan.is_too_early());
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
    async fn it_identifies_a_ucan_that_has_become_active() {
        let identities = Identities::new().await;
        let ucan = UcanBuilder::default()
            .issued_by(&identities.alice_key)
            .for_audience(identities.bob_did.as_str())
            .not_before(now() / 1000)
            .with_lifetime(30)
            .build()
            .unwrap()
            .sign()
            .await
            .unwrap();

        assert!(!ucan.is_too_early());
    }

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
    async fn it_can_be_serialized_as_json() {
        let identities = Identities::new().await;
        let ucan = UcanBuilder::default()
            .issued_by(&identities.alice_key)
            .for_audience(identities.bob_did.as_str())
            .not_before(now() / 1000)
            .with_lifetime(30)
            .build()
            .unwrap()
            .sign()
            .await
            .unwrap();

        let ucan_json = serde_json::to_value(ucan.clone()).unwrap();

        assert_eq!(
            ucan_json,
            serde_json::json!({
                "header": {
                    "alg": "EdDSA",
                    "typ": "JWT",
                    "ucv": crate::ucan::UCAN_VERSION
                },
                "payload": {
                    "iss": ucan.issuer(),
                    "aud": ucan.audience(),
                    "exp": ucan.expires_at(),
                    "nbf": ucan.not_before(),
                    "att": [],
                    "fct": [],
                    "prf": []
                },
                "signed_data": ucan.signed_data(),
                "signature": ucan.signature()
            })
        );
    }
}
