use crate::{
    capsule::Capsule,
    crypto::varsig,
    did::{Did, Verifiable},
};
use libipld_core::{
    cid::Cid,
    codec::{Codec, Encode},
    error::Result,
    ipld::Ipld,
    multihash::{Code, MultihashDigest},
};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use signature::{SignatureEncoding, Signer};
use std::collections::BTreeMap;
use thiserror::Error;

/// A container associating a `payload` with its signature over it.
#[derive(Debug, Clone, PartialEq)]
pub struct Envelope<
    T: Verifiable<DID> + Capsule,
    DID: Did,
    V: varsig::Header<Enc>,
    Enc: Codec + TryFrom<u32> + Into<u32>,
> {
    /// The [Varsig][crate::crypto::varsig] header.
    pub varsig_header: V,

    /// The signture of the `payload`.
    pub signature: DID::Signature,

    /// The payload that's being signed over.
    pub payload: T,

    _phantom: std::marker::PhantomData<Enc>,
}

impl<
        T: Verifiable<DID> + Capsule,
        DID: Did,
        V: varsig::Header<Enc>,
        Enc: Codec + TryFrom<u32> + Into<u32>,
    > Verifiable<DID> for Envelope<T, DID, V, Enc>
{
    fn verifier(&self) -> &DID {
        &self.payload.verifier()
    }
}

impl<
        T: Capsule + Verifiable<DID> + Into<Ipld>,
        DID: Did,
        V: varsig::Header<Enc>,
        Enc: Codec + TryFrom<u32> + Into<u32>,
    > Envelope<T, DID, V, Enc>
{
    pub fn new(varsig_header: V, signature: DID::Signature, payload: T) -> Self {
        Envelope {
            varsig_header,
            signature,
            payload,
            _phantom: std::marker::PhantomData,
        }
    }

    pub fn varsig_encode(self, w: &mut Vec<u8>) -> Result<(), libipld_core::error::Error>
    where
        Ipld: Encode<Enc> + From<Self>,
    {
        let codec = self.varsig_header.codec().clone();
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
    pub fn try_sign(
        signer: &DID::Signer,
        varsig_header: V,
        payload: T,
    ) -> Result<Envelope<T, DID, V, Enc>, SignError>
    where
        T: Clone,
        Ipld: Encode<Enc>,
    {
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
    /// FIXME
    pub fn try_sign_generic(
        signer: &DID::Signer,
        varsig_header: V,
        payload: T,
    ) -> Result<Envelope<T, DID, V, Enc>, SignError>
    where
        T: Clone,
        Ipld: Encode<Enc>,
    {
        let ipld: Ipld = BTreeMap::from_iter([(T::TAG.into(), payload.clone().into())]).into();

        let mut buffer = vec![];
        ipld.encode(*varsig_header.codec(), &mut buffer)
            .map_err(SignError::PayloadEncodingError)?;

        let signature = signer
            .try_sign(&buffer)
            .map_err(SignError::SignatureError)?;

        Ok(Envelope {
            varsig_header,
            signature,
            payload,
            _phantom: std::marker::PhantomData,
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
    pub fn validate_signature(&self) -> Result<(), ValidateError>
    where
        T: Clone,
        Ipld: Encode<Enc>,
    {
        let mut encoded = vec![];
        let ipld: Ipld = BTreeMap::from_iter([(T::TAG.into(), self.payload.clone().into())]).into();
        ipld.encode(*self.varsig_header.codec(), &mut encoded)
            .map_err(ValidateError::PayloadEncodingError)?;

        self.verifier()
            .verify(&encoded, &self.signature)
            .map_err(ValidateError::VerifyError)
    }

    pub fn cid(&self) -> Result<Cid, libipld_core::error::Error>
    where
        Self: Clone,
        Ipld: Encode<Enc> + From<T>,
    {
        let codec = self.varsig_header.codec().clone();
        let mut ipld_buffer = vec![];
        self.encode(codec, &mut ipld_buffer)?;

        let multihash = Code::Sha2_256.digest(&ipld_buffer);
        Ok(Cid::new_v1(
            self.varsig_header.codec().clone().into(),
            multihash,
        ))
    }
}

impl<
        T: Verifiable<DID> + Capsule,
        DID: Did,
        V: varsig::Header<Enc>,
        Enc: Codec + Into<u32> + TryFrom<u32>,
    > From<Envelope<T, DID, V, Enc>> for Ipld
where
    Ipld: From<T>,
{
    fn from(envelope: Envelope<T, DID, V, Enc>) -> Self {
        let ipld: Ipld = BTreeMap::from_iter([(T::TAG.into(), envelope.payload.into())]).into();
        let varsig_header: Ipld = Ipld::Bytes(envelope.varsig_header.into());

        Ipld::Map(BTreeMap::from_iter([
            ("sig".into(), Ipld::Bytes(envelope.signature.to_vec())),
            ("pld".into(), Ipld::List(vec![varsig_header, ipld])),
        ]))
    }
}

impl<
        T: TryFrom<Ipld> + Verifiable<DID> + Capsule,
        DID: Did,
        V: varsig::Header<Enc>,
        Enc: Codec + Into<u32> + TryFrom<u32>,
    > TryFrom<Ipld> for Envelope<T, DID, V, Enc>
{
    type Error = FromIpldError<T::Error>;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        if let Ipld::List(list) = ipld {
            if let [Ipld::Bytes(sig), Ipld::List(inner)] = list.as_slice() {
                if let [Ipld::Bytes(varsig_header), Ipld::Map(btree)] = inner.as_slice() {
                    if let (1, Some(payload)) = (btree.len(), btree.get(T::TAG.into())) {
                        Ok(Envelope {
                            payload: T::try_from(payload.clone())
                                .map_err(FromIpldError::CannotParsePayload)?,

                            varsig_header: V::try_from(varsig_header.as_slice())
                                .map_err(|_| FromIpldError::CannotParseVarsigHeader)?,

                            signature: DID::Signature::try_from(sig.as_slice())
                                .map_err(|_| FromIpldError::CannotParseSignature)?,

                            _phantom: std::marker::PhantomData,
                        })
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

impl<
        T: Verifiable<DID> + Capsule,
        DID: Did,
        V: varsig::Header<Enc>,
        Enc: Codec + Into<u32> + TryFrom<u32>,
    > Encode<Enc> for Envelope<T, DID, V, Enc>
where
    Self: Clone,
    Ipld: Encode<Enc> + From<T>,
{
    fn encode<W: std::io::Write>(
        &self,
        codec: Enc,
        w: &mut W,
    ) -> Result<(), libipld_core::error::Error> {
        let ipld: Ipld = self.clone().into();
        ipld.encode(codec, w)
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

impl<
        T: Verifiable<DID> + Capsule,
        DID: Did,
        V: varsig::Header<Enc>,
        Enc: Codec + TryFrom<u32> + Into<u32>,
    > Serialize for Envelope<T, DID, V, Enc>
{
    fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.clone().serialize(serializer)
    }
}

impl<
        'de,
        T: Verifiable<DID> + Capsule + TryFrom<Ipld>,
        DID: Did,
        V: varsig::Header<Enc>,
        Enc: Codec + TryFrom<u32> + Into<u32>,
    > Deserialize<'de> for Envelope<T, DID, V, Enc>
where
    Envelope<T, DID, V, Enc>: TryFrom<Ipld>,
    <Envelope<T, DID, V, Enc> as TryFrom<Ipld>>::Error: std::fmt::Display,
{
    fn deserialize<D>(deserializer: D) -> std::prelude::v1::Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let ipld: Ipld = Deserialize::deserialize(deserializer)?;
        Ok(Envelope::try_from(ipld).map_err(serde::de::Error::custom)?)
    }
}
