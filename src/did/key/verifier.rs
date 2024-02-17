use super::Signature;
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
use crate::crypto::es512;

#[cfg(feature = "rs256")]
use crate::crypto::rs256;

#[cfg(feature = "rs512")]
use crate::crypto::rs512;

#[cfg(feature = "bls")]
use blst;

/// Verifiers (public/verifying keys) for `did:key`.
#[derive(Debug, Clone, PartialEq, Eq, EnumAsInner)]
pub enum Verifier {
    /// `EdDSA` verifying key.
    #[cfg(feature = "eddsa")]
    EdDSA(ed25519_dalek::VerifyingKey),

    /// `ES256K` (`secp256k1`) verifying key.
    #[cfg(feature = "es256k")]
    Es256k(k256::ecdsa::VerifyingKey),

    /// `P-256` verifying key.
    #[cfg(feature = "es256")]
    P256(p256::ecdsa::VerifyingKey),

    /// `P-384` verifying key.
    #[cfg(feature = "es384")]
    P384(p384::ecdsa::VerifyingKey),

    /// `P-521` verifying key.
    #[cfg(feature = "es512")]
    P521(es512::VerifyingKey),

    /// `RS256` verifying key.
    #[cfg(feature = "rs256")]
    Rs256(rs256::VerifyingKey),

    /// `RS512` verifying key.
    #[cfg(feature = "rs512")]
    Rs512(rs512::VerifyingKey),

    /// `BLS 12-381` verifying key for the "min pub key" variant.
    #[cfg(feature = "bls")]
    BlsMinPk(blst::min_pk::PublicKey),

    /// `BLS 12-381` verifying key for the "min sig" variant.
    #[cfg(feature = "bls")]
    BlsMinSig(blst::min_sig::PublicKey),
}

impl signature::Verifier<Signature> for Verifier {
    fn verify(&self, msg: &[u8], signature: &Signature) -> Result<(), signature::Error> {
        match (self, signature) {
            (Verifier::EdDSA(vk), Signature::EdDSA(sig)) => {
                vk.verify(msg, sig).map_err(signature::Error::from_source)
            }
            (Verifier::Es256k(vk), Signature::Es256k(sig)) => {
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
