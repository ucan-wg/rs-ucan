mod did_from_keypair {
    use did_key::{Bls12381KeyPairs, Ed25519KeyPair, Generate, KeyPair};

    use crate::crypto::KeyMaterial;

    #[cfg(target_arch = "wasm32")]
    use wasm_bindgen_test::{wasm_bindgen_test, wasm_bindgen_test_configure};

    #[cfg(target_arch = "wasm32")]
    wasm_bindgen_test_configure!(run_in_browser);

    #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
    async fn it_handles_ed25519_keys() {
        let pub_key = base64::decode("Hv+AVRD2WUjUFOsSNbsmrp9fokuwrUnjBcr92f0kxw4=").unwrap();
        let keypair = KeyPair::Ed25519(Ed25519KeyPair::from_public_key(&pub_key));

        let expected_did = "did:key:z6MkgYGF3thn8k1Fv4p4dWXKtsXCnLH7q9yw4QgNPULDmDKB";
        let result_did = keypair.get_did().await.unwrap();

        assert_eq!(expected_did, result_did.as_str());
    }

    // #[cfg_attr(target_arch = "wasm32", wasm_bindgen_test)]
    #[ignore = "Public key is allegedly invalid size"]
    #[cfg_attr(not(target_arch = "wasm32"), tokio::test)]
    async fn it_handles_bls12381_keys() {
        let pub_key = base64::decode("Hv+AVRD2WUjUFOsSNbsmrp9fokuwrUnjBcr92f0kxw4=").unwrap();
        let keypair = KeyPair::Bls12381G1G2(Bls12381KeyPairs::from_public_key(&pub_key));

        let expected_did = "did:key:z6HpYD1br5P4QVh5rjRGAkBfKMWes44uhKmKdJ6dN2Nm9gHK";
        let result_did = keypair.get_did().await.unwrap();

        assert_eq!(expected_did, result_did.as_str());
    }
}
