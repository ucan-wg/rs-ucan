use super::fixtures::{EmailSemantics, Identities, SUPPORTED_KEYS};
use crate::{
    builder::UcanBuilder,
    capability::CapabilitySemantics,
    chain::{CapabilityInfo, ProofChain},
    crypto::did::DidParser,
    store::{MemoryStore, UcanJwtStore},
};
use std::collections::BTreeSet;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

#[cfg(target_arch = "wasm32")]
wasm_bindgen_test_configure!(run_in_browser);

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
pub async fn it_works_with_a_simple_example() {
    let identities = Identities::new().await;
    let did_parser = DidParser::new(SUPPORTED_KEYS);

    let email_semantics = EmailSemantics {};
    let send_email_as_alice = email_semantics
        .parse("mailto:alice@email.com", "email/send")
        .unwrap();

    let leaf_ucan = UcanBuilder::default()
        .issued_by(&identities.alice_key)
        .for_audience(identities.bob_did.as_str())
        .with_lifetime(60)
        .claiming_capability(&send_email_as_alice)
        .build()
        .unwrap()
        .sign()
        .await
        .unwrap();

    let attenuated_token = UcanBuilder::default()
        .issued_by(&identities.bob_key)
        .for_audience(identities.mallory_did.as_str())
        .with_lifetime(50)
        .witnessed_by(&leaf_ucan)
        .claiming_capability(&send_email_as_alice)
        .build()
        .unwrap()
        .sign()
        .await
        .unwrap()
        .encode()
        .unwrap();

    let mut store = MemoryStore::default();
    store
        .write_token(&leaf_ucan.encode().unwrap())
        .await
        .unwrap();

    let chain = ProofChain::try_from(attenuated_token.as_str())
        .unwrap()
        .with_parser(did_parser)
        .await
        .unwrap()
        .with_store(&store)
        .build()
        .await
        .unwrap();

    let capability_infos = chain.reduce_capabilities(&email_semantics);

    assert_eq!(capability_infos.len(), 1);

    let info = capability_infos.get(0).unwrap();

    assert_eq!(
        info.capability.with().to_string().as_str(),
        "mailto:alice@email.com",
    );
    assert_eq!(info.capability.can().to_string().as_str(), "email/send");
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
pub async fn it_reports_the_first_issuer_in_the_chain_as_originator() {
    let identities = Identities::new().await;
    let did_parser = DidParser::new(SUPPORTED_KEYS);

    let email_semantics = EmailSemantics {};
    let send_email_as_bob = email_semantics
        .parse("mailto:bob@email.com", "email/send")
        .unwrap();

    let leaf_ucan = UcanBuilder::default()
        .issued_by(&identities.alice_key)
        .for_audience(identities.bob_did.as_str())
        .with_lifetime(60)
        .build()
        .unwrap()
        .sign()
        .await
        .unwrap();

    let ucan_token = UcanBuilder::default()
        .issued_by(&identities.bob_key)
        .for_audience(identities.mallory_did.as_str())
        .with_lifetime(50)
        .witnessed_by(&leaf_ucan)
        .claiming_capability(&send_email_as_bob)
        .build()
        .unwrap()
        .sign()
        .await
        .unwrap()
        .encode()
        .unwrap();

    let mut store = MemoryStore::default();
    store
        .write_token(&leaf_ucan.encode().unwrap())
        .await
        .unwrap();

    let capability_infos = ProofChain::try_from(ucan_token)
        .unwrap()
        .with_parser(did_parser)
        .await
        .unwrap()
        .with_store(&store)
        .build()
        .await
        .unwrap()
        .reduce_capabilities(&email_semantics);

    assert_eq!(capability_infos.len(), 1);

    let info = capability_infos.get(0).unwrap();

    assert_eq!(
        info.originators.iter().collect::<Vec<&String>>(),
        vec![&identities.bob_did]
    );
    assert_eq!(info.capability, send_email_as_bob);
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
pub async fn it_finds_the_right_proof_chain_for_the_originator() {
    let identities = Identities::new().await;
    let did_parser = DidParser::new(SUPPORTED_KEYS);

    let email_semantics = EmailSemantics {};
    let send_email_as_bob = email_semantics
        .parse("mailto:bob@email.com", "email/send")
        .unwrap();
    let send_email_as_alice = email_semantics
        .parse("mailto:alice@email.com", "email/send")
        .unwrap();

    let leaf_ucan_alice = UcanBuilder::default()
        .issued_by(&identities.alice_key)
        .for_audience(identities.mallory_did.as_str())
        .with_lifetime(60)
        .claiming_capability(&send_email_as_alice)
        .build()
        .unwrap()
        .sign()
        .await
        .unwrap();

    let leaf_ucan_bob = UcanBuilder::default()
        .issued_by(&identities.bob_key)
        .for_audience(identities.mallory_did.as_str())
        .with_lifetime(60)
        .claiming_capability(&send_email_as_bob)
        .build()
        .unwrap()
        .sign()
        .await
        .unwrap();

    let ucan = UcanBuilder::default()
        .issued_by(&identities.mallory_key)
        .for_audience(identities.alice_did.as_str())
        .with_lifetime(50)
        .witnessed_by(&leaf_ucan_alice)
        .witnessed_by(&leaf_ucan_bob)
        .claiming_capability(&send_email_as_alice)
        .claiming_capability(&send_email_as_bob)
        .build()
        .unwrap()
        .sign()
        .await
        .unwrap();

    let ucan_token = ucan.encode().unwrap();

    let mut store = MemoryStore::default();
    store
        .write_token(&leaf_ucan_alice.encode().unwrap())
        .await
        .unwrap();
    store
        .write_token(&leaf_ucan_bob.encode().unwrap())
        .await
        .unwrap();

    let proof_chain = ProofChain::try_from(ucan_token.as_str())
        .unwrap()
        .with_parser(did_parser)
        .await
        .unwrap()
        .with_store(&store)
        .build()
        .await
        .unwrap();

    let capability_infos = proof_chain.reduce_capabilities(&email_semantics);

    assert_eq!(capability_infos.len(), 2);

    let send_email_as_bob_info = capability_infos.get(0).unwrap();
    let send_email_as_alice_info = capability_infos.get(1).unwrap();

    assert_eq!(
        send_email_as_alice_info,
        &CapabilityInfo {
            originators: BTreeSet::from_iter(vec![identities.alice_did]),
            capability: send_email_as_alice,
            not_before: *ucan.not_before(),
            expires_at: *ucan.expires_at()
        }
    );

    assert_eq!(
        send_email_as_bob_info,
        &CapabilityInfo {
            originators: BTreeSet::from_iter(vec![identities.bob_did]),
            capability: send_email_as_bob,
            not_before: *ucan.not_before(),
            expires_at: *ucan.expires_at()
        }
    );
}

#[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
#[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
pub async fn it_reports_all_chain_options() {
    let identities = Identities::new().await;
    let did_parser = DidParser::new(SUPPORTED_KEYS);

    let email_semantics = EmailSemantics {};
    let send_email_as_alice = email_semantics
        .parse("mailto:alice@email.com", "email/send")
        .unwrap();

    let leaf_ucan_alice = UcanBuilder::default()
        .issued_by(&identities.alice_key)
        .for_audience(identities.mallory_did.as_str())
        .with_lifetime(60)
        .claiming_capability(&send_email_as_alice)
        .build()
        .unwrap()
        .sign()
        .await
        .unwrap();

    let leaf_ucan_bob = UcanBuilder::default()
        .issued_by(&identities.bob_key)
        .for_audience(identities.mallory_did.as_str())
        .with_lifetime(60)
        .claiming_capability(&send_email_as_alice)
        .build()
        .unwrap()
        .sign()
        .await
        .unwrap();

    let ucan = UcanBuilder::default()
        .issued_by(&identities.mallory_key)
        .for_audience(identities.alice_did.as_str())
        .with_lifetime(50)
        .witnessed_by(&leaf_ucan_alice)
        .witnessed_by(&leaf_ucan_bob)
        .claiming_capability(&send_email_as_alice)
        .build()
        .unwrap()
        .sign()
        .await
        .unwrap();

    let ucan_token = ucan.encode().unwrap();

    let mut store = MemoryStore::default();
    store
        .write_token(&leaf_ucan_alice.encode().unwrap())
        .await
        .unwrap();
    store
        .write_token(&leaf_ucan_bob.encode().unwrap())
        .await
        .unwrap();

    let proof_chain = ProofChain::try_from(ucan_token)
        .unwrap()
        .with_parser(did_parser)
        .await
        .unwrap()
        .with_store(&store)
        .build()
        .await
        .unwrap();

    let capability_infos = proof_chain.reduce_capabilities(&email_semantics);

    assert_eq!(capability_infos.len(), 1);

    let send_email_as_alice_info = capability_infos.get(0).unwrap();

    assert_eq!(
        send_email_as_alice_info,
        &CapabilityInfo {
            originators: BTreeSet::from_iter(vec![identities.alice_did, identities.bob_did]),
            capability: send_email_as_alice,
            not_before: *ucan.not_before(),
            expires_at: *ucan.expires_at()
        }
    );
}
