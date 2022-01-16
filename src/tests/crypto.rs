mod did_from_keypair {
    use did_key::{Bls12381KeyPairs, Ed25519KeyPair};

    use crate::{crypto::did_from_keypair, crypto::Generate};

    #[test]
    #[ignore = "RSA key support not implemented"]
    fn it_handles_rsa_keys() {}

    #[test]
    fn it_handles_ed25519_keys() {
        let pub_key = base64::decode("Hv+AVRD2WUjUFOsSNbsmrp9fokuwrUnjBcr92f0kxw4=").unwrap();
        let keypair = Ed25519KeyPair::from_public_key(&pub_key);

        let expected_did = "did:key:z6MkgYGF3thn8k1Fv4p4dWXKtsXCnLH7q9yw4QgNPULDmDKB";
        let result_did = did_from_keypair(&keypair);

        assert_eq!(expected_did, result_did.as_str());
    }

    #[test]
    #[ignore = "Public key is allegedly invalid size"]
    fn it_handles_bls12381_keys() {
        let pub_key = base64::decode("Hv+AVRD2WUjUFOsSNbsmrp9fokuwrUnjBcr92f0kxw4=").unwrap();
        let keypair = Bls12381KeyPairs::from_public_key(&pub_key);

        let expected_did = "did:key:z6HpYD1br5P4QVh5rjRGAkBfKMWes44uhKmKdJ6dN2Nm9gHK";
        let result_did = did_from_keypair(&keypair);

        assert_eq!(expected_did, result_did.as_str());
    }
}
