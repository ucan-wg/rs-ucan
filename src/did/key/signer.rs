use super::Signature;
use enum_as_inner::EnumAsInner;

#[cfg(feature = "eddsa")]
use ed25519_dalek;

#[cfg(feature = "es256")]
use p256;

#[cfg(feature = "es256k")]
use k256;

#[cfg(feature = "es384")]
use p384;

#[cfg(feature = "es512")]
use ::p521 as ext_p521;

#[cfg(feature = "rs256")]
use crate::crypto::rs256;

#[cfg(feature = "rs512")]
use crate::crypto::rs512;

/// Signer types that are verifiable by `did:key` [`Verifier`]s.
#[derive(Clone, EnumAsInner)]
pub enum Signer {
    /// `EdDSA` signer.
    #[cfg(feature = "eddsa")]
    EdDsa(ed25519_dalek::SigningKey),

    /// `ES256K` (`secp256k1`) signer.
    #[cfg(feature = "es256k")]
    Es256k(k256::ecdsa::SigningKey),

    /// `P-256` signer.
    #[cfg(feature = "es256")]
    P256(p256::ecdsa::SigningKey),

    /// `P-384` signer.
    #[cfg(feature = "es384")]
    P384(p384::ecdsa::SigningKey),

    /// `P-521` signer.
    #[cfg(feature = "es512")]
    P521(ext_p521::ecdsa::SigningKey),

    /// `RS256` signer.
    #[cfg(feature = "rs256")]
    Rs256(rs256::SigningKey),

    /// `RS512` signer.
    #[cfg(feature = "rs512")]
    Rs512(rs512::SigningKey),

    /// `BLS 12-381` signer for the "min pub key" variant.
    #[cfg(feature = "bls")]
    BlsMinPk(blst::min_pk::SecretKey),

    /// `BLS 12-381` signer for the "min sig" variant.
    #[cfg(feature = "bls")]
    BlsMinSig(blst::min_sig::SecretKey),
}

impl signature::Signer<Signature> for Signer {
    fn try_sign(&self, msg: &[u8]) -> Result<Signature, signature::Error> {
        match self {
            #[cfg(feature = "eddsa")]
            Signer::EdDsa(signer) => {
                let sig = signer.sign(msg);
                Ok(Signature::EdDsa(sig))
            }

            #[cfg(feature = "es256k")]
            Signer::Es256k(signer) => {
                let sig = signer.sign(msg);
                Ok(Signature::Es256k(sig))
            }

            #[cfg(feature = "es256")]
            Signer::P256(signer) => {
                let sig = signer.sign(msg);
                Ok(Signature::P256(sig))
            }

            #[cfg(feature = "es384")]
            Signer::P384(signer) => {
                let sig = signer.sign(msg);
                Ok(Signature::P384(sig))
            }

            #[cfg(feature = "es512")]
            Signer::P521(signer) => {
                let sig = signer.sign(msg);
                Ok(Signature::P521(sig))
            }

            #[cfg(feature = "rs256")]
            Signer::Rs256(signer) => {
                let sig = signer.sign(msg);
                Ok(Signature::Rs256(sig))
            }

            #[cfg(feature = "rs512")]
            Signer::Rs512(signer) => {
                let sig = signer.sign(msg);
                Ok(Signature::Rs512(sig))
            }

            #[cfg(feature = "bls")]
            Signer::BlsMinPk(signer) => {
                let sig = signer.try_sign(msg)?;
                Ok(Signature::BlsMinPk(sig))
            }

            #[cfg(feature = "bls")]
            Signer::BlsMinSig(signer) => {
                let sig = signer.try_sign(msg)?;
                Ok(Signature::BlsMinSig(sig))
            }
        }
    }
}
