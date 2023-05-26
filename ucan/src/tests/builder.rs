use crate::{
    builder::UcanBuilder,
    capability::{CapabilityIpld, CapabilitySemantics},
    chain::ProofChain,
    crypto::did::DidParser,
    store::UcanJwtStore,
    tests::fixtures::{
        Blake2bMemoryStore, EmailSemantics, Identities, WNFSSemantics, SUPPORTED_KEYS,
    },
    time::now,
};
use cid::multihash::Code;
use did_key::PatchedKeyPair;
use serde_json::json;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test_configure!(run_in_browser);

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
async fn it_builds_with_a_simple_example() {
    let identities = Identities::new().await;

    let fact_1 = json!({
        "test": true
    });

    let fact_2 = json!({
        "preimage": "abc",
        "hash": "sth"
    });

    let email_semantics = EmailSemantics {};
    let wnfs_semantics = WNFSSemantics {};

    let cap_1 = email_semantics
        .parse("mailto:alice@gmail.com", "email/send")
        .unwrap();

    let cap_2 = wnfs_semantics
        .parse("wnfs://alice.fission.name/public", "wnfs/super_user")
        .unwrap();

    let expiration = now() + 30;
    let not_before = now() - 30;

    let token = UcanBuilder::default()
        .issued_by(&identities.alice_key)
        .for_audience(identities.bob_did.as_str())
        .with_expiration(expiration)
        .not_before(not_before)
        .with_fact(fact_1.clone())
        .with_fact(fact_2.clone())
        .claiming_capability(&cap_1)
        .claiming_capability(&cap_2)
        .with_nonce()
        .build()
        .unwrap();

    let ucan = token.sign().await.unwrap();

    assert_eq!(ucan.issuer(), identities.alice_did);
    assert_eq!(ucan.audience(), identities.bob_did);
    assert_eq!(ucan.expires_at(), &expiration);
    assert!(ucan.not_before().is_some());
    assert_eq!(ucan.not_before().unwrap(), not_before);
    assert_eq!(ucan.facts(), &Some(vec![fact_1, fact_2]));

    let expected_attenuations =
        Vec::from([CapabilityIpld::from(&cap_1), CapabilityIpld::from(&cap_2)]);

    assert_eq!(ucan.attenuation(), &expected_attenuations);
    assert!(ucan.nonce().is_some());
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
async fn it_builds_with_lifetime_in_seconds() {
    let identities = Identities::new().await;

    let ucan = UcanBuilder::default()
        .issued_by(&identities.alice_key)
        .for_audience(identities.bob_did.as_str())
        .with_lifetime(300)
        .build()
        .unwrap()
        .sign()
        .await
        .unwrap();

    assert!(*ucan.expires_at() > (now() + 290));
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
async fn it_prevents_duplicate_proofs() {
    let wnfs_semantics = WNFSSemantics {};

    let parent_cap = wnfs_semantics
        .parse("wnfs://alice.fission.name/public", "wnfs/super_user")
        .unwrap();

    let identities = Identities::new().await;
    let ucan = UcanBuilder::default()
        .issued_by(&identities.alice_key)
        .for_audience(identities.bob_did.as_str())
        .with_lifetime(30)
        .claiming_capability(&parent_cap)
        .build()
        .unwrap()
        .sign()
        .await
        .unwrap();

    let attenuated_cap_1 = wnfs_semantics
        .parse("wnfs://alice.fission.name/public/Apps", "wnfs/create")
        .unwrap();

    let attenuated_cap_2 = wnfs_semantics
        .parse("wnfs://alice.fission.name/public/Domains", "wnfs/create")
        .unwrap();

    let next_ucan = UcanBuilder::default()
        .issued_by(&identities.bob_key)
        .for_audience(identities.mallory_did.as_str())
        .with_lifetime(30)
        .witnessed_by(&ucan, None)
        .claiming_capability(&attenuated_cap_1)
        .claiming_capability(&attenuated_cap_2)
        .build()
        .unwrap()
        .sign()
        .await
        .unwrap();

    assert_eq!(
        next_ucan.proofs(),
        &Some(vec![ucan
            .to_cid(UcanBuilder::<PatchedKeyPair>::default_hasher())
            .unwrap()
            .to_string()])
    )
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
pub async fn it_can_use_custom_hasher() {
    let identities = Identities::new().await;
    let mut did_parser = DidParser::new(SUPPORTED_KEYS);

    let leaf_ucan = UcanBuilder::default()
        .issued_by(&identities.alice_key)
        .for_audience(identities.bob_did.as_str())
        .with_lifetime(60)
        .build()
        .unwrap()
        .sign()
        .await
        .unwrap();

    let delegated_token = UcanBuilder::default()
        .issued_by(&identities.alice_key)
        .issued_by(&identities.bob_key)
        .for_audience(identities.mallory_did.as_str())
        .with_lifetime(50)
        .witnessed_by(&leaf_ucan, Some(Code::Blake2b256))
        .build()
        .unwrap()
        .sign()
        .await
        .unwrap();

    let mut store = Blake2bMemoryStore::default();

    store
        .write_token(&leaf_ucan.encode().unwrap())
        .await
        .unwrap();

    let _ = store
        .write_token(&delegated_token.encode().unwrap())
        .await
        .unwrap();

    let valid_chain =
        ProofChain::from_ucan(delegated_token, Some(now()), &mut did_parser, &store).await;

    assert!(valid_chain.is_ok());
}
