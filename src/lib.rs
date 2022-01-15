#[macro_use]
extern crate log;

mod attenuation;
mod chain;
mod crypto;
mod time;
mod token;
mod types;
mod ucan;

pub use chain::ProofChain;
pub use token::{Token, TokenBuilder};
pub use ucan::Ucan;

// pub use token::TokenBuilder;

#[cfg(test)]
mod tests {

    use crate::crypto::{did_from_keypair, generate, Ed25519KeyPair, Generate, KeyPair};
    struct Fixtures {
        pub alice_key: KeyPair,
        pub bob_key: KeyPair,
        pub mallory_key: KeyPair,

        pub alice_did: String,
        pub bob_did: String,
        pub mallory_did: String,
    }

    impl Fixtures {
        pub fn new() -> Self {
            // let key_bytes = base64::decode("U+bzp2GaFQHso587iSFWPSeCzbSfn/CbNHEz7ilKRZ1UQMmMS7qq4UhTzKn3X9Nj/4xgrwa+UqhMOeo4Ki8JUw==".as_bytes()).unwrap();
            // println!("{:?}, {}", key_bytes, key_bytes.len());

            let alice_key = KeyPair::from(Ed25519KeyPair::from_secret_key(&base64::decode("U+bzp2GaFQHso587iSFWPSeCzbSfn/CbNHEz7ilKRZ1UQMmMS7qq4UhTzKn3X9Nj/4xgrwa+UqhMOeo4Ki8JUw==".as_bytes()).unwrap().as_slice()[0..32]));
            let bob_key = KeyPair::from(Ed25519KeyPair::from_secret_key(&base64::decode("G4+QCX1b3a45IzQsQd4gFMMe0UB1UOx9bCsh8uOiKLER69eAvVXvc8P2yc4Iig42Bv7JD2zJxhyFALyTKBHipg==".as_bytes()).unwrap().as_slice()[0..32]));
            let mallory_key = KeyPair::from(Ed25519KeyPair::from_secret_key(&base64::decode("LR9AL2MYkMARuvmV3MJV8sKvbSOdBtpggFCW8K62oZDR6UViSXdSV/dDcD8S9xVjS61vh62JITx7qmLgfQUSZQ==".as_bytes()).unwrap().as_slice()[0..32]));

            Fixtures {
                alice_did: did_from_keypair(&alice_key),
                bob_did: did_from_keypair(&bob_key),
                mallory_did: did_from_keypair(&mallory_key),

                alice_key,
                bob_key,
                mallory_key,
            }
        }

        pub fn name_for(&self, did: String) -> String {
            match did {
                _ if did == self.alice_did => "alice".into(),
                _ if did == self.bob_did => "bob".into(),
                _ if did == self.mallory_did => "mallory".into(),
                _ => did,
            }
        }
    }

    use crate::token::TokenBuilder;

    // builder.test.ts
    mod builder {
        use crate::{
            tests::Fixtures,
            time::now,
            types::{Capability, Fact},
            TokenBuilder,
        };
        use serde_json::{from_value, json, Value};

        #[test]
        fn it_builds_with_a_simple_example() {
            let fixtures = Fixtures::new();

            let fact1: Fact = from_value(json!({
                "test": true
            }))
            .unwrap();

            let fact2: Fact = from_value(json!({
                "preimage": "abc",
                "hash": "sth"
            }))
            .unwrap();

            let cap1: Capability = from_value(json!({
                "email": "alice@email.com",
                "cap": "SEND"
            }))
            .unwrap();

            let cap2: Capability = from_value(json!({
                "wnfs": "alice.fission.name/public/",
                "cap": "SUPER_USER"
            }))
            .unwrap();

            let expiration = now() + 30;
            let not_before = now() - 30;

            let token = TokenBuilder::new()
                .issued_by(&fixtures.alice_key)
                .to_audience(fixtures.bob_did.as_str())
                .with_expiration(expiration)
                .not_before(not_before)
                .with_fact(fact1.clone())
                .with_fact(fact2.clone())
                .claim_capability(cap1.clone())
                .claim_capability(cap2.clone())
                .with_nonce()
                .build()
                .unwrap();

            let ucan = token.sign().unwrap();

            assert_eq!(*ucan.issuer(), fixtures.alice_did);
            assert_eq!(*ucan.audience(), fixtures.bob_did);
            assert_eq!(*ucan.expires_at(), expiration);
            assert!(ucan.not_before().is_some());
            assert_eq!(ucan.not_before().unwrap(), not_before);
            assert_eq!(*ucan.facts(), Vec::from([fact1, fact2]));
            assert_eq!(*ucan.attenuation(), Vec::from([cap1, cap2]));
            assert!(ucan.nonce().is_some());
        }
    }
}
