mod validate {
    use crate::{
        builder::UcanBuilder,
        crypto::did::DidParser,
        tests::fixtures::{Identities, SUPPORTED_KEYS},
        time::now,
        ucan::Ucan,
    };

    #[tokio::test]
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
        let decoded_ucan = Ucan::try_from_token_string(encoded_ucan.as_str()).unwrap();

        decoded_ucan.validate(&mut did_parser).await.unwrap();
    }

    #[tokio::test]
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

    #[tokio::test]
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
}
