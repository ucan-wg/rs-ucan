use crate::{builder::UcanBuilder, chain::ProofChain, crypto::did::DidParser};

use super::fixtures::{Identities, SUPPORTED_KEYS};

#[tokio::test]
pub async fn it_decodes_deep_ucan_chains() {
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
        .issued_by(&identities.bob_key)
        .for_audience(identities.mallory_did.as_str())
        .with_lifetime(50)
        .witnessed_by(&leaf_ucan)
        .build()
        .unwrap()
        .sign()
        .await
        .unwrap()
        .encode()
        .unwrap();

    let chain = ProofChain::try_from_token_string(delegated_token.as_str(), &mut did_parser)
        .await
        .unwrap();

    assert_eq!(chain.ucan().audience(), &identities.mallory_did);
    assert_eq!(
        chain.proofs().get(0).unwrap().ucan().issuer(),
        &identities.alice_did
    );
}

#[tokio::test]
pub async fn it_fails_with_incorrect_chaining() {
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
        .for_audience(identities.mallory_did.as_str())
        .with_lifetime(50)
        .witnessed_by(&leaf_ucan)
        .build()
        .unwrap()
        .sign()
        .await
        .unwrap()
        .encode()
        .unwrap();

    let parse_token_result =
        ProofChain::try_from_token_string(delegated_token.as_str(), &mut did_parser).await;

    assert!(parse_token_result.is_err());
}

#[tokio::test]
pub async fn it_can_handle_multiple_leaves() {
    let identities = Identities::new().await;
    let mut did_parser = DidParser::new(SUPPORTED_KEYS);

    let leaf_ucan_1 = UcanBuilder::default()
        .issued_by(&identities.alice_key)
        .for_audience(identities.bob_did.as_str())
        .with_lifetime(60)
        .build()
        .unwrap()
        .sign()
        .await
        .unwrap();

    let leaf_ucan_2 = UcanBuilder::default()
        .issued_by(&identities.mallory_key)
        .for_audience(identities.bob_did.as_str())
        .with_lifetime(60)
        .build()
        .unwrap()
        .sign()
        .await
        .unwrap();

    let delegated_token = UcanBuilder::default()
        .issued_by(&identities.bob_key)
        .for_audience(identities.alice_did.as_str())
        .with_lifetime(50)
        .witnessed_by(&leaf_ucan_1)
        .witnessed_by(&leaf_ucan_2)
        .build()
        .unwrap()
        .sign()
        .await
        .unwrap()
        .encode()
        .unwrap();

    ProofChain::try_from_token_string(&delegated_token, &mut did_parser)
        .await
        .unwrap();
}
