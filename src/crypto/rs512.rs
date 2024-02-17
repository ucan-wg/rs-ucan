//! RS512 signature support (4096-bit RSA PKCS #1 v1.5).

use rsa;
use signature::{SignatureEncoding, Signer, Verifier};

/// The verifying/public key for RS512.
#[derive(Debug, Clone)] // FIXME , Serialize, Deserialize)]
pub struct VerifyingKey(pub rsa::pkcs1v15::VerifyingKey<rsa::sha2::Sha512>);

impl PartialEq for VerifyingKey {
    fn eq(&self, other: &Self) -> bool {
        rsa::RsaPublicKey::from(self.0.clone()) == rsa::RsaPublicKey::from(other.0.clone())
    }
}

impl Eq for VerifyingKey {}

impl Verifier<Signature> for VerifyingKey {
    fn verify(&self, msg: &[u8], signature: &Signature) -> Result<(), signature::Error> {
        self.0.verify(msg, &signature.0)
    }
}

/// The signing/secret key for RS512.
#[derive(Debug, Clone)] // FIXME , Serialize, Deserialize)]
pub struct SigningKey(pub rsa::pkcs1v15::SigningKey<rsa::sha2::Sha512>);

impl Signer<Signature> for SigningKey {
    fn try_sign(&self, msg: &[u8]) -> Result<Signature, signature::Error> {
        self.0.try_sign(msg).map(Signature)
    }
}

/// The signature for RS512.
#[derive(Debug, Clone, PartialEq, Eq)] // FIXME , Serialize, Deserialize)]
pub struct Signature(pub rsa::pkcs1v15::Signature);

impl SignatureEncoding for Signature {
    type Repr = [u8; 512];
}

impl From<[u8; 512]> for Signature {
    fn from(bytes: [u8; 512]) -> Self {
        Signature(
            rsa::pkcs1v15::Signature::try_from(bytes.as_ref())
                .expect("passed in [u8; 512], so should succeed"),
        )
    }
}

impl From<Signature> for [u8; 512] {
    fn from(sig: Signature) -> [u8; 512] {
        sig.0
            .to_bytes()
            .as_ref()
            .try_into()
            .expect("Signature should be exactly 512 bytes")
    }
}

impl<'a> TryFrom<&'a [u8]> for Signature {
    type Error = signature::Error;

    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        rsa::pkcs1v15::Signature::try_from(bytes).map(Signature)
    }
}
