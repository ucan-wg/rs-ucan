use super::Witness;
use crate::{
    capsule::Capsule,
    did::{Did, Verifiable},
};
use libipld_core::{
    codec::{Codec, Encode},
    error::Result,
    ipld::Ipld,
    multihash::Code,
};
use std::collections::BTreeMap;
use thiserror::Error;

// FIXME #[cfg(feature = "dag-cbor")]
use libipld_cbor::DagCborCodec;
use signature::Signer;

/// A container associating a `payload` with its signature over it.
#[derive(Debug, Clone, PartialEq)] // , Serialize, Deserialize)]
pub struct Envelope<T: Verifiable<DID> + Capsule, DID: Did> {
    /// The signture of the `payload`.
    pub signature: Witness<DID::Signature>,

    /// The payload that's being signed over.
    pub payload: T,
}

impl<T: Verifiable<DID> + Capsule, DID: Did> Verifiable<DID> for Envelope<T, DID> {
    fn verifier(&self) -> &DID {
        &self.payload.verifier()
    }
}

impl<T: Capsule + Verifiable<DID> + Into<Ipld> + Clone, DID: Did> Envelope<T, DID> {
    /// Attempt to sign some payload with a given signer.
    ///
    /// # Arguments
    ///
    /// * `signer` - The signer to use to sign the payload.
    /// * `payload` - The payload to sign.
    ///
    /// # Errors
    ///
    /// * [`SignError`] - the payload can't be encoded or the signature fails.
    ///
    /// # Example
    ///
    /// FIXME
    pub fn try_sign(signer: &DID::Signer, payload: T) -> Result<Envelope<T, DID>, SignError> {
        Self::try_sign_generic::<DagCborCodec, Code>(signer, DagCborCodec, payload)
    }

    /// Attempt to sign some payload with a given signer and specific codec.
    ///
    /// # Arguments
    ///
    /// * `signer` - The signer to use to sign the payload.
    /// * `codec` - The codec to use to encode the payload.
    /// * `payload` - The payload to sign.
    ///
    /// # Errors
    ///
    /// * [`SignError`] - the payload can't be encoded or the signature fails.
    ///
    /// # Example
    ///
    /// FIXME
    pub fn try_sign_generic<C: Codec, H: Into<u64>>(
        signer: &DID::Signer,
        codec: C,
        payload: T,
    ) -> Result<Envelope<T, DID>, SignError>
    where
        Ipld: Encode<C>,
    {
        let ipld: Ipld = BTreeMap::from_iter([(T::TAG.into(), payload.clone().into())]).into();

        let mut buffer = vec![];
        ipld.encode(codec, &mut buffer)
            .map_err(SignError::PayloadEncodingError)?;

        let sig = signer
            .try_sign(&buffer)
            .map_err(SignError::SignatureError)?;

        Ok(Envelope {
            signature: Witness::Signature(sig),
            payload,
        })
    }

    /// Attempt to validate a signature.
    ///
    /// # Arguments
    ///
    /// * `self` - The envelope to validate.
    ///
    /// # Errors
    ///
    /// * [`ValidateError`] - the payload can't be encoded or the signature fails.
    ///
    /// # Exmaples
    ///
    /// FIXME
    pub fn validate_signature(&self) -> Result<(), ValidateError> {
        // FIXME need varsig
        let codec = DagCborCodec;

        let mut encoded = vec![];
        let ipld: Ipld = BTreeMap::from_iter([(T::TAG.into(), self.payload.clone().into())]).into();
        ipld.encode(codec, &mut encoded)
            .map_err(ValidateError::PayloadEncodingError)?;

        match &self.signature {
            Witness::Signature(sig) => self
                .verifier()
                .verify(&encoded, &sig)
                .map_err(ValidateError::VerifyError),
        }
    }
}

impl<T: Verifiable<DID> + Capsule + Into<Ipld>, DID: Did> From<Envelope<T, DID>> for Ipld {
    fn from(Envelope { signature, payload }: Envelope<T, DID>) -> Self {
        let ipld: Ipld = BTreeMap::from_iter([(T::TAG.into(), payload.into())]).into();
        BTreeMap::from_iter([("sig".into(), signature.into()), ("pld".into(), ipld)]).into()
    }
}

/// Errors that can occur when signing a [`siganture::Envelope`][Envelope].
#[derive(Debug, Error)]
pub enum SignError {
    /// Unable to encode the payload.
    #[error("Unable to encode payload")]
    PayloadEncodingError(#[from] libipld_core::error::Error),

    /// Error while signing.
    #[error("Signature error: {0}")]
    SignatureError(#[from] signature::Error),
}

/// Errors that can occur when validating a [`signature::Envelope`][Envelope].
#[derive(Debug, Error)]
pub enum ValidateError {
    /// Unable to encode the payload.
    #[error("Unable to encode payload")]
    PayloadEncodingError(#[from] libipld_core::error::Error),

    /// Error while verifying the signature.
    #[error("Signature verification failed: {0}")]
    VerifyError(#[from] signature::Error),
}
