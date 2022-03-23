use did_key::{Ed25519KeyPair, Generate, KeyPair};

use crate::crypto::KeyMaterial;

pub struct Identities {
    pub alice_key: KeyPair,
    pub bob_key: KeyPair,
    pub mallory_key: KeyPair,

    pub alice_did: String,
    pub bob_did: String,
    pub mallory_did: String,
}

/// An adaptation of the fixtures used in the canonical ts-ucan repo
/// See: https://github.com/ucan-wg/ts-ucan/blob/main/tests/fixtures.ts
impl Identities {
    pub async fn new() -> Self {
        // NOTE: tweetnacl secret keys concat the public keys, so we only care
        // about the first 32 bytes
        let alice_key = KeyPair::from(Ed25519KeyPair::from_secret_key(&base64::decode("U+bzp2GaFQHso587iSFWPSeCzbSfn/CbNHEz7ilKRZ1UQMmMS7qq4UhTzKn3X9Nj/4xgrwa+UqhMOeo4Ki8JUw==".as_bytes()).unwrap().as_slice()[0..32]));
        let bob_key = KeyPair::from(Ed25519KeyPair::from_secret_key(&base64::decode("G4+QCX1b3a45IzQsQd4gFMMe0UB1UOx9bCsh8uOiKLER69eAvVXvc8P2yc4Iig42Bv7JD2zJxhyFALyTKBHipg==".as_bytes()).unwrap().as_slice()[0..32]));
        let mallory_key = KeyPair::from(Ed25519KeyPair::from_secret_key(&base64::decode("LR9AL2MYkMARuvmV3MJV8sKvbSOdBtpggFCW8K62oZDR6UViSXdSV/dDcD8S9xVjS61vh62JITx7qmLgfQUSZQ==".as_bytes()).unwrap().as_slice()[0..32]));

        Identities {
            alice_did: alice_key.get_did().await.unwrap(),
            bob_did: bob_key.get_did().await.unwrap(),
            mallory_did: mallory_key.get_did().await.unwrap(),

            alice_key,
            bob_key,
            mallory_key,
        }
    }

    #[allow(dead_code)]
    pub fn name_for(&self, did: String) -> String {
        match did {
            _ if did == self.alice_did => "alice".into(),
            _ if did == self.bob_did => "bob".into(),
            _ if did == self.mallory_did => "mallory".into(),
            _ => did,
        }
    }
}
