use super::fixtures::Identities;
use crate::{
    capability::{proof::ProofDelegationSemantics, CapabilitySemantics},
    ProofChain, UcanBuilder,
};

#[test]
pub fn it_decodes_deep_ucan_chains() {
    let identities = Identities::new();
    let leaf_ucan = UcanBuilder::new()
        .issued_by(&identities.alice_key)
        .for_audience(identities.bob_did.as_str())
        .with_lifetime(60)
        .build()
        .unwrap()
        .sign()
        .unwrap();

    let delegated_token = UcanBuilder::new()
        .issued_by(&identities.bob_key)
        .for_audience(identities.mallory_did.as_str())
        .with_lifetime(50)
        .with_proof(&leaf_ucan)
        .build()
        .unwrap()
        .sign()
        .unwrap()
        .encode()
        .unwrap();

    let chain = ProofChain::from_token_string(delegated_token.as_str()).unwrap();

    assert_eq!(chain.ucan().audience(), &identities.mallory_did);
    assert_eq!(
        chain.proofs().get(0).unwrap().ucan().issuer(),
        &identities.alice_did
    );
}

#[test]
pub fn it_fails_with_incorrect_chaining() {
    let identities = Identities::new();
    let leaf_ucan = UcanBuilder::new()
        .issued_by(&identities.alice_key)
        .for_audience(identities.bob_did.as_str())
        .with_lifetime(60)
        .build()
        .unwrap()
        .sign()
        .unwrap();

    let delegated_token = UcanBuilder::new()
        .issued_by(&identities.alice_key)
        .for_audience(identities.mallory_did.as_str())
        .with_lifetime(50)
        .with_proof(&leaf_ucan)
        .build()
        .unwrap()
        .sign()
        .unwrap()
        .encode()
        .unwrap();

    let parse_token_result = ProofChain::from_token_string(delegated_token.as_str());

    assert!(parse_token_result.is_err());
}

#[test]
pub fn it_can_handle_multiple_leaves() {
    let identities = Identities::new();
    let leaf_ucan_1 = UcanBuilder::new()
        .issued_by(&identities.alice_key)
        .for_audience(identities.bob_did.as_str())
        .with_lifetime(60)
        .build()
        .unwrap()
        .sign()
        .unwrap();

    let leaf_ucan_2 = UcanBuilder::new()
        .issued_by(&identities.mallory_key)
        .for_audience(identities.bob_did.as_str())
        .with_lifetime(60)
        .build()
        .unwrap()
        .sign()
        .unwrap();

    let delegated_token = UcanBuilder::new()
        .issued_by(&identities.bob_key)
        .for_audience(identities.alice_did.as_str())
        .with_lifetime(50)
        .with_proof(&leaf_ucan_1)
        .with_proof(&leaf_ucan_2)
        .build()
        .unwrap()
        .sign()
        .unwrap()
        .encode()
        .unwrap();

    ProofChain::from_token_string(&delegated_token).unwrap();
}
