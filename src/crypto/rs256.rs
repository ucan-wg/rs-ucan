//! RS256 signature support

use rsa;
use signature::{SignatureEncoding, Signer, Verifier};

#[derive(Debug, Clone)] // FIXME , Serialize, Deserialize)]
pub struct VerifyingKey(pub rsa::pkcs1v15::VerifyingKey<rsa::sha2::Sha256>);

impl PartialEq for VerifyingKey {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_ref() == other.0.as_ref()
    }
}

impl Eq for VerifyingKey {}

impl Verifier<Signature> for VerifyingKey {
    fn verify(&self, msg: &[u8], signature: &Signature) -> Result<(), signature::Error> {
        self.0.verify(msg, &signature.0)
    }
}

#[derive(Debug, Clone)] // FIXME , Serialize, Deserialize)]
pub struct SigningKey(pub rsa::pkcs1v15::SigningKey<rsa::sha2::Sha256>);

impl Signer<Signature> for SigningKey {
    fn try_sign(&self, msg: &[u8]) -> Result<Signature, signature::Error> {
        self.0.try_sign(msg).map(Signature)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)] // FIXME , Serialize, Deserialize)]
pub struct Signature(pub rsa::pkcs1v15::Signature);

impl SignatureEncoding for Signature {
    type Repr = [u8; 256];
}

impl From<[u8; 256]> for Signature {
    fn from(bytes: [u8; 256]) -> Self {
        Signature(
            rsa::pkcs1v15::Signature::try_from(bytes.as_ref())
                .expect("passed in [u8; 256], so should succeed"),
        )
    }
}

impl From<Signature> for [u8; 256] {
    fn from(sig: Signature) -> [u8; 256] {
        sig.0
            .to_bytes()
            .as_ref()
            .try_into()
            .expect("Signature should be exactly 256 bytes")
    }
}

impl<'a> TryFrom<&'a [u8]> for Signature {
    type Error = signature::Error;

    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        rsa::pkcs1v15::Signature::try_from(bytes).map(Signature)
    }
}
