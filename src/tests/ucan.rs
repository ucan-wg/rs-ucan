mod validate {
    use crate::{tests::fixtures::Identities, Ucan, UcanBuilder};

    #[test]
    fn it_round_trips_with_encode() {
        let identities = Identities::new();
        let ucan = UcanBuilder::new()
            .issued_by(&identities.alice_key)
            .to_audience(identities.bob_did.as_str())
            .with_lifetime(30)
            .build()
            .unwrap()
            .sign()
            .unwrap();

        let encoded_ucan = ucan.encoded().unwrap();
        let decoded_ucan = Ucan::from_token_string(encoded_ucan.as_str()).unwrap();

        decoded_ucan.validate().unwrap();
    }
}
