//! Tests for delegation conformance to the UCAN specification.
mod delegation_conformance {
    use std::sync::OnceLock;

    use base64::prelude::*;
    use testresult::TestResult;
    use ucan::{did::Ed25519Did, Delegation};

    const DELEGATION_FIXTURE_STR: &str = include_str!("./fixtures/delegation.json");
    static DELEGATION_FIXTURE: OnceLock<serde_json::Value> = OnceLock::new();
    fn delegation_fixture() -> &'static serde_json::Value {
        DELEGATION_FIXTURE.get_or_init(|| {
            serde_json::from_str(DELEGATION_FIXTURE_STR).expect("fixture is invalid JSON")
        })
    }

    #[test]
    fn test_expected_version() -> TestResult {
        assert_eq!(
            delegation_fixture()
                .get("version")
                .expect("fixture has delegation key")
                .clone(),
            "1.0.0-rc.1".to_string()
        );
        Ok(())
    }

    #[test]
    fn test_top_level_parse() -> TestResult {
        let b64_txt: &str = delegation_fixture()["valid"][0]["token"]
            .as_str()
            .expect("valid delegation token is a string");

        let bytes: Vec<u8> = BASE64_STANDARD.decode(b64_txt)?;
        let delegation: Delegation<Ed25519Did> = serde_ipld_dagcbor::from_slice(&bytes)?; //
        assert_eq!(delegation.policy(), &vec![]);

        Ok(())
    }
}
