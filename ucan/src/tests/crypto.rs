mod did_from_keypair {
    use did_key::{Bls12381KeyPairs, Ed25519KeyPair, Generate, KeyPair};

    use crate::crypto::KeyMaterial;

    #[cfg(feature = "rsa_support")]
    #[test]
    fn it_handles_rsa_keys() {
        use crate::crypto::rsa::RsaKeyPair;
        use rsa::pkcs8::FromPublicKey;
        use rsa::RsaPublicKey;

        let pub_key = base64::decode("MIIBIjANBgkqhkiG9w0BAQEFAAOCAQ8AMIIBCgKCAQEAnzyis1ZjfNB0bBgKFMSvvkTtwlvBsaJq7S5wA+kzeVOVpVWwkWdVha4s38XM/pa/yr47av7+z3VTmvDRyAHcaT92whREFpLv9cj5lTeJSibyr/Mrm/YtjCZVWgaOYIhwrXwKLqPr/11inWsAkfIytvHWTxZYEcXLgAXFuUuaS3uF9gEiNQwzGTU1v0FqkqTBr4B8nW3HCN47XUu0t8Y0e+lf4s4OxQawWD79J9/5d3Ry0vbV3Am1FtGJiJvOwRsIfVChDpYStTcHTCMqtvWbV6L11BWkpzGXSW4Hv43qa+GSYOD2QU68Mb59oSk2OB+BtOLpJofmbGEGgvmwyCI9MwIDAQAB").unwrap();
        let keypair = RsaKeyPair(
            RsaPublicKey::from_public_key_der(pub_key.as_slice()).unwrap(),
            None,
        );

        let expected_did = "did:key:z4MXj1wBzi9jUstyNvmiK5WLRRL4rr9UvzPxhry1CudCLKWLyMbP1WoTwDfttBTpxDKf5hAJEjqNbeYx2EEvrJmSWHAu7TJRPTrE3QodbMfRvRNRDyYvaN1FSQus2ziS1rWXwAi5Gpc16bY3JwjyLCPJLfdRWHZhRXiay5FWEkfoSKy6aftnzAvqNkKBg2AxgzGMinR6d1WiH4w5mEXFtUeZkeo4uwtRTd8rD9BoVaHVkGwJkksDybE23CsBNXiNfbweFVRcwfTMhcQsTsYhUWDcSC6QE3zt9h4Rsrj7XRYdwYSK5bc1qFRsg5HULKBp2uZ1gcayiW2FqHFcMRjBieC4LnSMSD1AZB1WUncVRbPpVkn1UGhCU";
        let result_did = keypair.get_did();

        assert_eq!(expected_did, result_did.as_str());
    }

    #[tokio::test]
    async fn it_handles_ed25519_keys() {
        let pub_key = base64::decode("Hv+AVRD2WUjUFOsSNbsmrp9fokuwrUnjBcr92f0kxw4=").unwrap();
        let keypair = KeyPair::Ed25519(Ed25519KeyPair::from_public_key(&pub_key));

        let expected_did = "did:key:z6MkgYGF3thn8k1Fv4p4dWXKtsXCnLH7q9yw4QgNPULDmDKB";
        let result_did = keypair.get_did().await.unwrap();

        assert_eq!(expected_did, result_did.as_str());
    }

    #[tokio::test]
    #[ignore = "Public key is allegedly invalid size"]
    async fn it_handles_bls12381_keys() {
        let pub_key = base64::decode("Hv+AVRD2WUjUFOsSNbsmrp9fokuwrUnjBcr92f0kxw4=").unwrap();
        let keypair = KeyPair::Bls12381G1G2(Bls12381KeyPairs::from_public_key(&pub_key));

        let expected_did = "did:key:z6HpYD1br5P4QVh5rjRGAkBfKMWes44uhKmKdJ6dN2Nm9gHK";
        let result_did = keypair.get_did().await.unwrap();

        assert_eq!(expected_did, result_did.as_str());
    }
}
