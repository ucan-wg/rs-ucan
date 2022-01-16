use crate::{
    tests::fixtures::{EmailCapability, Identities, WNFSCapability},
    time::now,
    UcanBuilder,
};
use serde_json::json;

#[test]
fn it_builds_with_a_simple_example() {
    let identities = Identities::new();

    let fact_1 = json!({
        "test": true
    });

    let fact_2 = json!({
        "preimage": "abc",
        "hash": "sth"
    });

    let cap_1 = EmailCapability {
        email: "alice@gmail.com".into(),
        cap: "SEND".into(),
    };

    let cap_2 = WNFSCapability {
        wnfs: "alice.fission.name/public".into(),
        cap: "SUPER_USER".into(),
    };

    let expiration = now() + 30;
    let not_before = now() - 30;

    let token = UcanBuilder::new()
        .issued_by(&identities.alice_key)
        .to_audience(identities.bob_did.as_str())
        .with_expiration(expiration)
        .not_before(not_before)
        .with_fact(fact_1.clone())
        .with_fact(fact_2.clone())
        .claim_capability(cap_1.clone())
        .claim_capability(cap_2.clone())
        .with_nonce()
        .build()
        .unwrap();

    let ucan = token.sign().unwrap();

    assert_eq!(*ucan.issuer(), identities.alice_did);
    assert_eq!(*ucan.audience(), identities.bob_did);
    assert_eq!(*ucan.expires_at(), expiration);
    assert!(ucan.not_before().is_some());
    assert_eq!(ucan.not_before().unwrap(), not_before);
    assert_eq!(*ucan.facts(), Vec::from([fact_1, fact_2]));

    let expected_attenuations = Vec::from([
        serde_json::to_value(cap_1).unwrap(),
        serde_json::to_value(cap_2).unwrap(),
    ]);

    assert_eq!(*ucan.attenuation(), expected_attenuations);
    assert!(ucan.nonce().is_some());
}

#[test]
fn it_builds_with_lifetime_in_seconds() {
    let identities = Identities::new();

    let ucan = UcanBuilder::new()
        .issued_by(&identities.alice_key)
        .to_audience(identities.bob_did.as_str())
        .with_lifetime(300)
        .build()
        .unwrap()
        .sign()
        .unwrap();

    assert!(*ucan.expires_at() > (now() + 290));
}

#[test]
fn it_prevents_duplicate_proofs() {
    let parent_cap = WNFSCapability {
        wnfs: "alice.fission.name/public".into(),
        cap: "SUPER_USER".into(),
    };

    let identities = Identities::new();
    let ucan = UcanBuilder::new()
        .issued_by(&identities.alice_key)
        .to_audience(identities.bob_did.as_str())
        .with_lifetime(30)
        .claim_capability(parent_cap)
        .build()
        .unwrap()
        .sign()
        .unwrap();

    let attenuated_cap_1 = WNFSCapability {
        wnfs: "alice.fission.name/public/Apps".into(),
        cap: "CREATE".into(),
    };

    let attenuated_cap_2 = WNFSCapability {
        wnfs: "alice.fission.name/public/Documents".into(),
        cap: "OVERWRITE".into(),
    };

    let next_ucan = UcanBuilder::new()
        .issued_by(&identities.bob_key)
        .to_audience(identities.mallory_did.as_str())
        .with_lifetime(30)
        .delegate_capability(attenuated_cap_1, &ucan)
        .delegate_capability(attenuated_cap_2, &ucan)
        .build()
        .unwrap()
        .sign()
        .unwrap();

    assert_eq!(*next_ucan.proofs(), Vec::from([ucan.encoded().unwrap()]))
}
