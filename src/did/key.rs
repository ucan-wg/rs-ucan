//! Support for the `did:key` scheme

pub mod traits;

use signature;

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

#[cfg(feature = "es512")]
use ::p521 as ext_p521;

#[cfg(feature = "rs256")]
use crate::crypto::rs256;

#[cfg(feature = "rs512")]
use crate::crypto::rs512;

#[cfg(feature = "bls")]
use blst;

#[cfg(feature = "bls")]
use crate::crypto::bls12381;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Signature {
    #[cfg(feature = "eddsa")]
    Ed25519(ed25519_dalek::Signature),

    #[cfg(feature = "es256k")]
    Sedcp256k1(k256::ecdsa::Signature),

    #[cfg(feature = "es256")]
    P256(p256::ecdsa::Signature),

    #[cfg(feature = "es384")]
    P384(p384::ecdsa::Signature),

    #[cfg(feature = "es512")]
    P521(ext_p521::ecdsa::Signature),

    #[cfg(feature = "rs256")]
    Rs256(rs256::Signature),

    #[cfg(feature = "rs512")]
    Rs512(rs512::Signature),

    #[cfg(feature = "bls")]
    BlsMinPk(bls12381::min_pk::Signature),

    #[cfg(feature = "bls")]
    BlsMinSig(bls12381::min_sig::Signature),
}

impl signature::Verifier<Signature> for Verifier {
    fn verify(&self, msg: &[u8], signature: &Signature) -> Result<(), signature::Error> {
        match (self, signature) {
            (Verifier::Ed25519(vk), Signature::Ed25519(sig)) => {
                vk.verify(msg, sig).map_err(signature::Error::from_source)
            }
            (Verifier::Sedcp256k1(vk), Signature::Sedcp256k1(sig)) => {
                vk.verify(msg, sig).map_err(signature::Error::from_source)
            }
            (Verifier::P256(vk), Signature::P256(sig)) => {
                vk.verify(msg, sig).map_err(signature::Error::from_source)
            }
            (Verifier::P384(vk), Signature::P384(sig)) => {
                vk.verify(msg, sig).map_err(signature::Error::from_source)
            }
            (Verifier::P521(vk), Signature::P521(sig)) => {
                vk.verify(msg, sig).map_err(signature::Error::from_source)
            }
            (Verifier::Rs256(vk), Signature::Rs256(sig)) => {
                vk.verify(msg, sig).map_err(signature::Error::from_source)
            }
            (Verifier::Rs512(vk), Signature::Rs512(sig)) => {
                vk.verify(msg, sig).map_err(signature::Error::from_source)
            }
            (Verifier::BlsMinPk(vk), Signature::BlsMinPk(sig)) => {
                vk.verify(msg, sig).map_err(signature::Error::from_source)
            }
            (Verifier::BlsMinSig(vk), Signature::BlsMinSig(sig)) => {
                vk.verify(msg, sig).map_err(signature::Error::from_source)
            }
            (_, _) => Err(signature::Error::from_source(
                "invalid signature type for verifier",
            )),
        }
    }
}
