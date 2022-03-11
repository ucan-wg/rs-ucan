mod validate {
    use crate::{builder::UcanBuilder, tests::fixtures::Identities, time::now, ucan::Ucan};

    #[test]
    fn it_round_trips_with_encode() {
        let identities = Identities::new();
        let ucan = UcanBuilder::new()
            .issued_by(&identities.alice_key)
            .for_audience(identities.bob_did.as_str())
            .with_lifetime(30)
            .build()
            .unwrap()
            .sign()
            .unwrap();

        let encoded_ucan = ucan.encode().unwrap();
        let decoded_ucan = Ucan::from_token_string(encoded_ucan.as_str()).unwrap();

        decoded_ucan.validate().unwrap();
    }

    #[test]
    fn it_identifies_a_ucan_that_is_not_active_yet() {
        let identities = Identities::new();
        let ucan = UcanBuilder::new()
            .issued_by(&identities.alice_key)
            .for_audience(identities.bob_did.as_str())
            .not_before(now() + 30)
            .with_lifetime(30)
            .build()
            .unwrap()
            .sign()
            .unwrap();

        assert!(ucan.is_too_early());
    }

    #[test]
    fn it_identifies_a_ucan_that_has_become_active() {
        let identities = Identities::new();
        let ucan = UcanBuilder::new()
            .issued_by(&identities.alice_key)
            .for_audience(identities.bob_did.as_str())
            .not_before(now() / 1000)
            .with_lifetime(30)
            .build()
            .unwrap()
            .sign()
            .unwrap();

        assert!(!ucan.is_too_early());
    }
}
