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

#[cfg(feature = "bls")]
use crate::crypto::bls12381;

/// Signature types that are verifiable by `did:key` [`Verifier`]s.
#[derive(Debug, Clone, PartialEq, Eq, EnumAsInner)]
pub enum Signature {
    /// `EdDSA` signature.
    #[cfg(feature = "eddsa")]
    EdDsa(ed25519_dalek::Signature),

    /// `ES256K` (`secp256k1`) signature.
    #[cfg(feature = "es256k")]
    Es256k(k256::ecdsa::Signature),

    /// `P-256` signature.
    #[cfg(feature = "es256")]
    P256(p256::ecdsa::Signature),

    /// `P-384` signature.
    #[cfg(feature = "es384")]
    P384(p384::ecdsa::Signature),

    /// `P-521` signature.
    #[cfg(feature = "es512")]
    P521(ext_p521::ecdsa::Signature),

    /// `RS256` signature.
    #[cfg(feature = "rs256")]
    Rs256(rs256::Signature),

    /// `RS512` signature.
    #[cfg(feature = "rs512")]
    Rs512(rs512::Signature),

    /// `BLS 12-381` signature for the "min pub key" variant.
    #[cfg(feature = "bls")]
    BlsMinPk(bls12381::min_pk::Signature),

    /// `BLS 12-381` signature for the "min sig" variant.
    #[cfg(feature = "bls")]
    BlsMinSig(bls12381::min_sig::Signature),

    /// An unknown signature type.
    ///
    /// This is primarily for parsing, where reification is delayed
    /// until the DID method is known.
    Unknown(Vec<u8>),
}

impl signature::SignatureEncoding for Signature {
    type Repr = Vec<u8>;
}

impl From<Signature> for Vec<u8> {
    fn from(sig: Signature) -> Vec<u8> {
        match sig {
            #[cfg(feature = "eddsa")]
            Signature::EdDsa(sig) => sig.to_vec(),

            #[cfg(feature = "es256k")]
            Signature::Es256k(sig) => sig.to_vec(),

            #[cfg(feature = "es256")]
            Signature::P256(sig) => sig.to_vec(),

            #[cfg(feature = "es384")]
            Signature::P384(sig) => sig.to_vec(),

            #[cfg(feature = "es512")]
            Signature::P521(sig) => sig.to_vec(),

            #[cfg(feature = "rs256")]
            Signature::Rs256(sig) => <[u8; 256]>::from(sig).into(),

            #[cfg(feature = "rs512")]
            Signature::Rs512(sig) => <[u8; 512]>::from(sig).into(),

            #[cfg(feature = "bls")]
            Signature::BlsMinPk(sig) => <[u8; 96]>::from(sig).into(),

            #[cfg(feature = "bls")]
            Signature::BlsMinSig(sig) => <[u8; 48]>::from(sig).into(),

            Signature::Unknown(vec) => vec,
        }
    }
}

impl From<&[u8]> for Signature {
    fn from(arr: &[u8]) -> Signature {
        Signature::Unknown(arr.to_vec())
    }
}
