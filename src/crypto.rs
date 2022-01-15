use anyhow::{anyhow, Result};
pub use did_key::*;

/// Verify an alleged signature of some data given a DID
pub fn verify_signature(data: &Vec<u8>, signature: &Vec<u8>, did: &String) -> Result<()> {
    let did = did_key::resolve(did).map_err(|_| anyhow!("Failed to parse DID: {}", did))?;
    did.verify(data, signature)
        .map_err(|_| anyhow!("Signature could not be verified"))?;
    Ok(())
}

pub fn did_from_keypair<T: Fingerprint + Into<KeyPair>>(keypair: &T) -> String {
    format!("did:key:{}", keypair.fingerprint())
}

pub fn jwt_algorithm_for_keypair(keypair: &KeyPair) -> String {
    match keypair {
        KeyPair::Ed25519(_) => String::from("EdDSA"),
        _ => todo!(),
    }
}
