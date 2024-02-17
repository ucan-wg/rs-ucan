use enum_as_inner::EnumAsInner;
// FIXME use serde::{Deserialize, Serialize};

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
    EdDSA(ed25519_dalek::Signature),

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
}
