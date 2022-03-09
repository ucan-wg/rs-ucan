use super::fixtures::Identities;
use crate::{
    capability::{proof::ProofDelegationSemantics, CapabilitySemantics},
    ProofChain, UcanBuilder,
};

#[test]
pub fn it_decodes_deep_ucan_chains() {
    let identities = Identities::new();
    let root_ucan = UcanBuilder::new()
        .issued_by(&identities.alice_key)
        .for_audience(identities.bob_did.as_str())
        .with_lifetime(60)
        .build()
        .unwrap()
        .sign()
        .unwrap();

    let proof_delegation = ProofDelegationSemantics {};

    let delegated_token = UcanBuilder::new()
        .issued_by(&identities.bob_key)
        .for_audience(identities.mallory_did.as_str())
        .with_lifetime(50)
        .delegate_capability(
            proof_delegation
                .parse("prf:0".into(), "ucan/DELEGATE".into())
                .unwrap(),
            &root_ucan,
        )
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
