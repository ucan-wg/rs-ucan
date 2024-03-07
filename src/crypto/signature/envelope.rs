use crate::{capsule::Capsule, crypto::varsig, did::Did};
use libipld_core::{
    cid::Cid,
    codec::{Codec, Encode},
    error::Result,
    ipld::Ipld,
    multihash::{Code, MultihashDigest},
};
use signature::Verifier;
use signature::{SignatureEncoding, Signer};
use std::collections::BTreeMap;
use thiserror::Error;

pub trait Envelope: Sized {
    type DID: Did;
    type Payload: Clone + Capsule + TryFrom<Ipld> + Into<Ipld>;
    type VarsigHeader: varsig::Header<Self::Encoder> + Clone;
    type Encoder: Codec + TryFrom<u64> + Into<u64>;

    fn varsig_header(&self) -> &Self::VarsigHeader;
    fn signature(&self) -> &<Self::DID as Did>::Signature;
    fn payload(&self) -> &Self::Payload;
    fn verifier(&self) -> &Self::DID;

    fn construct(
        varsig_header: Self::VarsigHeader,
        signature: <Self::DID as Did>::Signature,
        payload: Self::Payload,
    ) -> Self;

    fn to_ipld_envelope(&self) -> Ipld {
        let wrapped_payload: Ipld =
            BTreeMap::from_iter([(Self::Payload::TAG.into(), self.payload().clone().into())])
                .into();

        let header_bytes: Vec<u8> = (*self.varsig_header()).clone().into();
        let header: Ipld = vec![header_bytes.into(), wrapped_payload].into();
        let sig_bytes: Ipld = self.signature().to_vec().into();

        vec![sig_bytes.into(), header].into()
    }

    fn try_from_ipld_envelope(
        ipld: Ipld,
    ) -> Result<Self, FromIpldError<<Self::Payload as TryFrom<Ipld>>::Error>> {
        if let Ipld::List(list) = ipld {
            if let [Ipld::Bytes(sig), Ipld::List(inner)] = list.as_slice() {
                if let [Ipld::Bytes(varsig_header), Ipld::Map(btree)] = inner.as_slice() {
                    if let (1, Some(inner)) = (
                        btree.len(),
                        btree.get(<Self::Payload as Capsule>::TAG.into()),
                    ) {
                        let payload = Self::Payload::try_from(inner.clone())
                            .map_err(FromIpldError::CannotParsePayload)?;

                        let varsig_header = Self::VarsigHeader::try_from(varsig_header.as_slice())
                            .map_err(|_| FromIpldError::CannotParseVarsigHeader)?;

                        let signature = <Self::DID as Did>::Signature::try_from(sig.as_slice())
                            .map_err(|_| FromIpldError::CannotParseSignature)?;

                        Ok(Self::construct(varsig_header, signature, payload))
                    } else {
                        Err(FromIpldError::InvalidPayloadCapsule)
                    }
                } else {
                    Err(FromIpldError::InvalidVarsigContainer)
                }
            } else {
                Err(FromIpldError::InvalidSignatureContainer)
            }
        } else {
            Err(FromIpldError::InvalidSignatureContainer)
        }
    }

    fn varsig_encode(self, w: &mut Vec<u8>) -> Result<(), libipld_core::error::Error>
    where
        Ipld: Encode<Self::Encoder> + From<Self>,
    {
        let codec = varsig::header::Header::codec(self.varsig_header()).clone();
        let ipld = Ipld::from(self);
        ipld.encode(codec, w)
    }

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
    // FIXME ported
    fn try_sign(
        signer: &<Self::DID as Did>::Signer,
        varsig_header: Self::VarsigHeader,
        payload: Self::Payload,
    ) -> Result<Self, SignError>
    where
        Ipld: Encode<Self::Encoder> + From<Self::Payload>, // FIXME force it to be named args not IPLD
    {
        dbg!("try_sign");
        Self::try_sign_generic(signer, varsig_header, payload)
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
    fn try_sign_generic(
        signer: &<Self::DID as Did>::Signer,
        varsig_header: Self::VarsigHeader,
        payload: Self::Payload,
    ) -> Result<Self, SignError>
    where
        Ipld: Encode<Self::Encoder> + From<Self::Payload>,
    {
        dbg!("try_sign_generic");
        let ipld: Ipld =
            BTreeMap::from_iter([(Self::Payload::TAG.into(), Ipld::from(payload.clone()))]).into();

        dbg!("buffer");
        let mut buffer = vec![];
        dbg!("encode");
        ipld.encode(*varsig::header::Header::codec(&varsig_header), &mut buffer)
            .map_err(SignError::PayloadEncodingError)?;

        dbg!("sign");
        let signature =
            signature::Signer::try_sign(signer, &buffer).map_err(SignError::SignatureError)?;

        dbg!("construct");
        Ok(Self::construct(varsig_header, signature, payload))
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
    fn validate_signature(&self) -> Result<(), ValidateError>
    where
        Ipld: Encode<Self::Encoder> + From<Self::Payload>,
    {
        let mut encoded = vec![];
        let ipld: Ipld =
            BTreeMap::from_iter([(Self::Payload::TAG.into(), self.payload().clone().into())])
                .into();
        ipld.encode(
            *varsig::header::Header::codec(self.varsig_header()),
            &mut encoded,
        )
        .map_err(ValidateError::PayloadEncodingError)?;

        self.verifier()
            .verify(&encoded, &self.signature())
            .map_err(ValidateError::VerifyError)
    }

    fn cid(&self) -> Result<Cid, libipld_core::error::Error>
    where
        // Ipld: Encode<Self::Encoder> + From<Self::Payload>,
        Self: Encode<Self::Encoder>,
    {
        let codec = varsig::header::Header::codec(self.varsig_header()).clone();
        let mut ipld_buffer = vec![];
        self.encode(codec, &mut ipld_buffer)?;

        let multihash = Code::Sha2_256.digest(&ipld_buffer);
        Ok(Cid::new_v1(
            varsig::header::Header::codec(self.varsig_header())
                .clone()
                .into(),
            multihash,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum FromIpldError<E> {
    #[error("Invalid signature container")]
    InvalidSignatureContainer,

    #[error("Invalid varsig container")]
    InvalidVarsigContainer,

    #[error("Cannot parse payload: {0}")]
    CannotParsePayload(#[from] E),

    #[error("Cannot parse varsig header")]
    CannotParseVarsigHeader,

    #[error("Cannot parse signature")]
    CannotParseSignature,

    #[error("Invalid payload capsule")]
    InvalidPayloadCapsule,
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
