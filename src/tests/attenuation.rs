use std::collections::BTreeSet;

use super::fixtures::{EmailSemantics, Identities, SUPPORTED_KEYS};
use crate::capability::CapabilitySemantics;
use crate::{
    builder::UcanBuilder,
    chain::{CapabilityInfo, ProofChain},
    crypto::did::DidParser,
};

#[test]
pub fn it_works_with_a_simple_example() {
    let identities = Identities::new();
    let did_parser = DidParser::new(SUPPORTED_KEYS);

    let email_semantics = EmailSemantics {};
    let send_email_as_alice = email_semantics
        .parse("mailto:alice@email.com".into(), "email/SEND".into())
        .unwrap();

    let leaf_ucan = UcanBuilder::new()
        .issued_by(&identities.alice_key)
        .for_audience(identities.bob_did.as_str())
        .with_lifetime(60)
        .claiming_capability(&send_email_as_alice)
        .build()
        .unwrap()
        .sign()
        .unwrap();

    let attenuated_token = UcanBuilder::new()
        .issued_by(&identities.bob_key)
        .for_audience(identities.mallory_did.as_str())
        .with_lifetime(50)
        .witnessed_by(&leaf_ucan)
        .claiming_capability(&send_email_as_alice)
        .build()
        .unwrap()
        .sign()
        .unwrap()
        .encode()
        .unwrap();

    let chain = ProofChain::try_from_token_string(attenuated_token.as_str(), &did_parser).unwrap();

    let capability_infos = chain.reduce_capabilities(&email_semantics);

    assert_eq!(capability_infos.len(), 1);

    let info = capability_infos.get(0).unwrap();

    assert_eq!(
        info.capability.with().to_string().as_str(),
        "mailto:alice@email.com",
    );
    assert_eq!(info.capability.can().to_string().as_str(), "email/SEND");
}

#[test]
pub fn it_reports_the_first_issuer_in_the_chain_as_originator() {
    let identities = Identities::new();
    let did_parser = DidParser::new(SUPPORTED_KEYS);

    let email_semantics = EmailSemantics {};
    let send_email_as_bob = email_semantics
        .parse("mailto:bob@email.com".into(), "email/SEND".into())
        .unwrap();

    let leaf_ucan = UcanBuilder::new()
        .issued_by(&identities.alice_key)
        .for_audience(identities.bob_did.as_str())
        .with_lifetime(60)
        .build()
        .unwrap()
        .sign()
        .unwrap();

    let ucan_token = UcanBuilder::new()
        .issued_by(&identities.bob_key)
        .for_audience(identities.mallory_did.as_str())
        .with_lifetime(50)
        .witnessed_by(&leaf_ucan)
        .claiming_capability(&send_email_as_bob)
        .build()
        .unwrap()
        .sign()
        .unwrap()
        .encode()
        .unwrap();

    let capability_infos = ProofChain::try_from_token_string(&ucan_token, &did_parser)
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

#[test]
pub fn it_finds_the_right_proof_chain_for_the_originator() {
    let identities = Identities::new();
    let did_parser = DidParser::new(SUPPORTED_KEYS);

    let email_semantics = EmailSemantics {};
    let send_email_as_bob = email_semantics
        .parse("mailto:bob@email.com".into(), "email/SEND".into())
        .unwrap();
    let send_email_as_alice = email_semantics
        .parse("mailto:alice@email.com".into(), "email/SEND".into())
        .unwrap();

    let leaf_ucan_alice = UcanBuilder::new()
        .issued_by(&identities.alice_key)
        .for_audience(identities.mallory_did.as_str())
        .with_lifetime(60)
        .claiming_capability(&send_email_as_alice)
        .build()
        .unwrap()
        .sign()
        .unwrap();

    let leaf_ucan_bob = UcanBuilder::new()
        .issued_by(&identities.bob_key)
        .for_audience(identities.mallory_did.as_str())
        .with_lifetime(60)
        .claiming_capability(&send_email_as_bob)
        .build()
        .unwrap()
        .sign()
        .unwrap();

    let ucan = UcanBuilder::new()
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
        .unwrap();

    let ucan_token = ucan.encode().unwrap();

    let proof_chain = ProofChain::try_from_token_string(&ucan_token, &did_parser).unwrap();
    let capability_infos = proof_chain.reduce_capabilities(&email_semantics);

    assert_eq!(capability_infos.len(), 2);

    let send_email_as_bob_info = capability_infos.get(0).unwrap();
    let send_email_as_alice_info = capability_infos.get(1).unwrap();

    assert_eq!(
        send_email_as_alice_info,
        &CapabilityInfo {
            originators: BTreeSet::from_iter(vec![identities.alice_did]),
            capability: send_email_as_alice,
            not_before: ucan.not_before().clone(),
            expires_at: ucan.expires_at().clone()
        }
    );

    assert_eq!(
        send_email_as_bob_info,
        &CapabilityInfo {
            originators: BTreeSet::from_iter(vec![identities.bob_did]),
            capability: send_email_as_bob,
            not_before: ucan.not_before().clone(),
            expires_at: ucan.expires_at().clone()
        }
    );
}

#[test]
pub fn it_reports_all_chain_options() {
    let identities = Identities::new();
    let did_parser = DidParser::new(SUPPORTED_KEYS);

    let email_semantics = EmailSemantics {};
    let send_email_as_alice = email_semantics
        .parse("mailto:alice@email.com".into(), "email/SEND".into())
        .unwrap();

    let leaf_ucan_alice = UcanBuilder::new()
        .issued_by(&identities.alice_key)
        .for_audience(identities.mallory_did.as_str())
        .with_lifetime(60)
        .claiming_capability(&send_email_as_alice)
        .build()
        .unwrap()
        .sign()
        .unwrap();

    let leaf_ucan_bob = UcanBuilder::new()
        .issued_by(&identities.bob_key)
        .for_audience(identities.mallory_did.as_str())
        .with_lifetime(60)
        .claiming_capability(&send_email_as_alice)
        .build()
        .unwrap()
        .sign()
        .unwrap();

    let ucan = UcanBuilder::new()
        .issued_by(&identities.mallory_key)
        .for_audience(identities.alice_did.as_str())
        .with_lifetime(50)
        .witnessed_by(&leaf_ucan_alice)
        .witnessed_by(&leaf_ucan_bob)
        .claiming_capability(&send_email_as_alice)
        .build()
        .unwrap()
        .sign()
        .unwrap();

    let ucan_token = ucan.encode().unwrap();

    let proof_chain = ProofChain::try_from_token_string(&ucan_token, &did_parser).unwrap();
    let capability_infos = proof_chain.reduce_capabilities(&email_semantics);

    assert_eq!(capability_infos.len(), 1);

    let send_email_as_alice_info = capability_infos.get(0).unwrap();

    assert_eq!(
        send_email_as_alice_info,
        &CapabilityInfo {
            originators: BTreeSet::from_iter(vec![identities.alice_did, identities.bob_did]),
            capability: send_email_as_alice,
            not_before: ucan.not_before().clone(),
            expires_at: ucan.expires_at().clone()
        }
    );
}
