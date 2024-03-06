use enum_as_inner::EnumAsInner;
use super::Signature;

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

#[cfg(feature = "bls")]
use crate::crypto::bls12381;



/// Signature types that are verifiable by `did:key` [`Verifier`]s.
#[derive(Debug, Clone, PartialEq, Eq, EnumAsInner)]
pub enum Signer {
    /// `EdDSA` signature.
    #[cfg(feature = "eddsa")]
    EdDsa(ed25519_dalek::SigningKey),

    // /// `ES256K` (`secp256k1`) signature.
    // #[cfg(feature = "es256k")]
    // Es256k(k256::ecdsa::Signer),

    // /// `P-256` signature.
    // #[cfg(feature = "es256")]
    // P256(p256::ecdsa::Signer),

    // /// `P-384` signature.
    // #[cfg(feature = "es384")]
    // P384(p384::ecdsa::Signer),

    // /// `P-521` signature.
    // #[cfg(feature = "es512")]
    // P521(ext_p521::ecdsa::Signer),

    // /// `RS256` signature.
    // #[cfg(feature = "rs256")]
    // Rs256(rs256::Signer),

    // /// `RS512` signature.
    // #[cfg(feature = "rs512")]
    // Rs512(rs512::Signer),

    // /// `BLS 12-381` signature for the "min pub key" variant.
    // #[cfg(feature = "bls")]
    // BlsMinPk(bls12381::min_pk::Signer),

    // /// `BLS 12-381` signature for the "min sig" variant.
    // #[cfg(feature = "bls")]
    // BlsMinSig(bls12381::min_sig::Signer),

    // /// An unknown signature type.
    // ///
    // /// This is primarily for parsing, where reification is delayed
    // /// until the DID method is known.
    // Unknown(Vec<u8>),
}

impl signature::Signer<Signature> for Signer {
    fn try_sign(&self, msg: &[u8]) -> Result<Signature, signature::Error> {
        match self {
            #[cfg(feature = "eddsa")]
            Signer::EdDsa(signer) => {
                let sig = signer.sign(msg);
                Ok(Signature::EdDsa(sig))
            }
        }
    }
}
