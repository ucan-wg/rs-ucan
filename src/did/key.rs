//! Support for the `did:key` scheme

pub mod traits;

#[cfg(feature = "eddsa")]
use ed25519_dalek;

#[cfg(feature = "es256")]
use p256;

#[cfg(feature = "es256k")]
use k256;

#[cfg(feature = "es384")]
use p384;

#[cfg(feature = "es512")]
use crate::crypto::p521;

#[cfg(feature = "rs256")]
use crate::crypto::rs256;

#[cfg(feature = "rs512")]
use crate::crypto::rs512;

#[cfg(feature = "bls")]
use blst;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Verifier {
    #[cfg(feature = "eddsa")]
    Ed25519(ed25519_dalek::VerifyingKey),

    #[cfg(feature = "es256k")]
    Sedcp256k1(k256::ecdsa::VerifyingKey),

    #[cfg(feature = "es256")]
    P256(p256::ecdsa::VerifyingKey),

    #[cfg(feature = "es384")]
    P384(p384::ecdsa::VerifyingKey),

    #[cfg(feature = "es512")]
    P521(p521::VerifyingKey),

    #[cfg(feature = "rs256")]
    Rs256(rs256::VerifyingKey),

    #[cfg(feature = "rs512")]
    Rs512(rs512::VerifyingKey),

    #[cfg(feature = "bls")]
    BlsMinPk(blst::min_pk::PublicKey),

    #[cfg(feature = "bls")]
    BlsMinSig(blst::min_sig::PublicKey),
}
