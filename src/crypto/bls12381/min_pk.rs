use super::error::VerificationError;
use crate::crypto::domain_separator::DomainSeparator;
use blst::BLST_ERROR;
use signature::{SignatureEncoding, Signer, Verifier};

/// A BLS12-381 MinPubKey signature
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signature(pub blst::min_pk::Signature);

impl DomainSeparator for Signature {
    /// From the [IETF BLS Signature Spec](https://www.ietf.org/archive/id/draft-irtf-cfrg-bls-signature-05.html#section-4.2.1)
    const DST: &'static [u8] = b"BLS_SIG_BLS12381G2_XMD:SHA-256_SSWU_RO_NUL_";
}

impl<'a> TryFrom<&'a [u8]> for Signature {
    type Error = BLST_ERROR;

    fn try_from(bytes: &'a [u8]) -> Result<Self, Self::Error> {
        Ok(Self(blst::min_pk::Signature::uncompress(bytes)?))
    }
}

impl From<Signature> for [u8; 96] {
    fn from(sig: Signature) -> Self {
        sig.0.compress()
    }
}

impl SignatureEncoding for Signature {
    type Repr = [u8; 96];
}

impl Signer<Signature> for blst::min_pk::SecretKey {
    fn try_sign(&self, msg: &[u8]) -> Result<Signature, signature::Error> {
        Ok(Signature(self.sign(msg, Signature::DST, &[])))
    }
}

impl Verifier<Signature> for blst::min_pk::PublicKey {
    fn verify(&self, msg: &[u8], signature: &Signature) -> Result<(), signature::Error> {
        match VerificationError::try_from(signature.0.verify(
            true,
            msg,
            Signature::DST,
            &[],
            &self,
            true,
        )) {
            Ok(err) => Err(signature::Error::from_source(err)),
            Err(_) => Ok(()),
        }
    }
}
