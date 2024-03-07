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
    type Encoder: Codec + TryFrom<u32> + Into<u32>;

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
        Ipld: Encode<Self::Encoder> + From<Self::Payload>,
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
    fn try_sign_generic(
        signer: &<Self::DID as Did>::Signer,
        varsig_header: Self::VarsigHeader,
        payload: Self::Payload,
    ) -> Result<Self, SignError>
    where
        Ipld: Encode<Self::Encoder> + From<Self::Payload>,
    {
        let ipld: Ipld =
            BTreeMap::from_iter([(Self::Payload::TAG.into(), payload.clone().into())]).into();

        let mut buffer = vec![];
        ipld.encode(*varsig::header::Header::codec(&varsig_header), &mut buffer)
            .map_err(SignError::PayloadEncodingError)?;

        let signature = signer
            .try_sign(&buffer)
            .map_err(SignError::SignatureError)?;

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

// /// A container associating a `payload` with its signature over it.
// #[derive(Debug, Clone, PartialEq)]
// pub struct Envelope<
//     T: Verifiable<DID> + Capsule,
//     DID: Did,
//     V: varsig::Header<C>,
//     C: Codec + TryFrom<u32> + Into<u32>,
// > {
//     /// The [Varsig][crate::crypto::varsig] header.
//     pub varsig_header: V,
//
//     /// The signture of the `payload`.
//     pub signature: DID::Signature,
//
//     /// The payload that's being signed over.
//     pub payload: T,
//
//     _phantom: std::marker::PhantomData<C>,
// }

// impl<
//         T: Verifiable<DID> + Capsule,
//         DID: Did,
//         V: varsig::Header<C>,
//         C: Codec + TryFrom<u32> + Into<u32>,
//     > Verifiable<DID> for Envelope<T, DID, V, C>
// {
//     fn verifier(&self) -> &DID {
//         &self.payload.verifier()
//     }
// }
//
// impl<
//         T: Capsule + Verifiable<DID> + Into<Ipld>,
//         DID: Did,
//         V: varsig::Header<C>,
//         C: Codec + TryFrom<u32> + Into<u32>,
//     > Envelope<T, DID, V, C>
// {
//     pub fn new(varsig_header: V, signature: DID::Signature, payload: T) -> Self {
//         Envelope {
//             varsig_header,
//             signature,
//             payload,
//             _phantom: std::marker::PhantomData,
//         }
//     }
//
//     // FIXME ported
//     pub fn varsig_encode(self, w: &mut Vec<u8>) -> Result<(), libipld_core::error::Error>
//     where
//         Ipld: Encode<C> + From<Self>,
//     {
//         let codec = self.varsig_header.codec().clone();
//         let ipld = Ipld::from(self);
//         ipld.encode(codec, w)
//     }
//
//     /// Attempt to sign some payload with a given signer.
//     ///
//     /// # Arguments
//     ///
//     /// * `signer` - The signer to use to sign the payload.
//     /// * `payload` - The payload to sign.
//     ///
//     /// # Errors
//     ///
//     /// * [`SignError`] - the payload can't be encoded or the signature fails.
//     // FIXME ported
//     pub fn try_sign(
//         signer: &DID::Signer,
//         varsig_header: V,
//         payload: T,
//     ) -> Result<Envelope<T, DID, V, C>, SignError>
//     where
//         T: Clone,
//         Ipld: Encode<C>,
//     {
//         Self::try_sign_generic(signer, varsig_header, payload)
//     }
//
//     /// Attempt to sign some payload with a given signer and specific codec.
//     ///
//     /// # Arguments
//     ///
//     /// * `signer` - The signer to use to sign the payload.
//     /// * `codec` - The codec to use to encode the payload.
//     /// * `payload` - The payload to sign.
//     ///
//     /// # Errors
//     ///
//     /// * [`SignError`] - the payload can't be encoded or the signature fails.
//     ///
//     /// # Example
//     ///
//     /// FIXME ported
//     pub fn try_sign_generic(
//         signer: &DID::Signer,
//         varsig_header: V,
//         payload: T,
//     ) -> Result<Envelope<T, DID, V, C>, SignError>
//     where
//         T: Clone,
//         Ipld: Encode<C>,
//     {
//         let ipld: Ipld = BTreeMap::from_iter([(T::TAG.into(), payload.clone().into())]).into();
//
//         let mut buffer = vec![];
//         ipld.encode(*varsig_header.codec(), &mut buffer)
//             .map_err(SignError::PayloadEncodingError)?;
//
//         let signature = signer
//             .try_sign(&buffer)
//             .map_err(SignError::SignatureError)?;
//
//         Ok(Envelope {
//             varsig_header,
//             signature,
//             payload,
//             _phantom: std::marker::PhantomData,
//         })
//     }
//
//     /// Attempt to validate a signature.
//     ///
//     /// # Arguments
//     ///
//     /// * `self` - The envelope to validate.
//     ///
//     /// # Errors
//     ///
//     /// * [`ValidateError`] - the payload can't be encoded or the signature fails.
//     ///
//     /// # Exmaples
//     ///
//     /// FIXME
//     pub fn validate_signature(&self) -> Result<(), ValidateError>
//     where
//         T: Clone,
//         Ipld: Encode<C>,
//     {
//         let mut encoded = vec![];
//         let ipld: Ipld = BTreeMap::from_iter([(T::TAG.into(), self.payload.clone().into())]).into();
//         ipld.encode(*self.varsig_header.codec(), &mut encoded)
//             .map_err(ValidateError::PayloadEncodingError)?;
//
//         self.verifier()
//             .verify(&encoded, &self.signature)
//             .map_err(ValidateError::VerifyError)
//     }
//
//     pub fn cid(&self) -> Result<Cid, libipld_core::error::Error>
//     where
//         Self: Clone,
//         Ipld: Encode<C> + From<T>,
//     {
//         let codec = self.varsig_header.codec().clone();
//         let mut ipld_buffer = vec![];
//         self.encode(codec, &mut ipld_buffer)?;
//
//         let multihash = Code::Sha2_256.digest(&ipld_buffer);
//         Ok(Cid::new_v1(
//             self.varsig_header.codec().clone().into(),
//             multihash,
//         ))
//     }
// }

// impl<
//         T: Verifiable<DID> + Capsule,
//         DID: Did,
//         V: varsig::Header<C>,
//         C: Codec + Into<u32> + TryFrom<u32>,
//     > From<Envelope<T, DID, V, C>> for Ipld
// where
//     Ipld: From<T>,
// {
//     fn from(envelope: Envelope<T, DID, V, C>) -> Self {
//         let ipld: Ipld = BTreeMap::from_iter([(T::TAG.into(), envelope.payload.into())]).into();
//         let varsig_header: Ipld = Ipld::Bytes(envelope.varsig_header.into());
//
//         Ipld::Map(BTreeMap::from_iter([
//             ("sig".into(), Ipld::Bytes(envelope.signature.to_vec())),
//             ("pld".into(), Ipld::List(vec![varsig_header, ipld])),
//         ]))
//     }
// }
//
// impl<
//         T: TryFrom<Ipld> + Verifiable<DID> + Capsule,
//         DID: Did,
//         V: varsig::Header<C>,
//         C: Codec + Into<u32> + TryFrom<u32>,
//     > TryFrom<Ipld> for Envelope<T, DID, V, C>
// {
//     type Error = FromIpldError<T::Error>;
//
//     fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
//         if let Ipld::List(list) = ipld {
//             if let [Ipld::Bytes(sig), Ipld::List(inner)] = list.as_slice() {
//                 if let [Ipld::Bytes(varsig_header), Ipld::Map(btree)] = inner.as_slice() {
//                     if let (1, Some(payload)) = (btree.len(), btree.get(T::TAG.into())) {
//                         Ok(Envelope {
//                             payload: T::try_from(payload.clone())
//                                 .map_err(FromIpldError::CannotParsePayload)?,
//
//                             varsig_header: V::try_from(varsig_header.as_slice())
//                                 .map_err(|_| FromIpldError::CannotParseVarsigHeader)?,
//
//                             signature: DID::Signature::try_from(sig.as_slice())
//                                 .map_err(|_| FromIpldError::CannotParseSignature)?,
//
//                             _phantom: std::marker::PhantomData,
//                         })
//                     } else {
//                         Err(FromIpldError::InvalidPayloadCapsule)
//                     }
//                 } else {
//                     Err(FromIpldError::InvalidVarsigContainer)
//                 }
//             } else {
//                 Err(FromIpldError::InvalidSignatureContainer)
//             }
//         } else {
//             Err(FromIpldError::InvalidSignatureContainer)
//         }
//     }
// }

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
//
// impl<
//         T: Verifiable<DID> + Capsule,
//         DID: Did,
//         V: varsig::Header<C>,
//         C: Codec + Into<u32> + TryFrom<u32>,
//     > Encode<C> for Envelope<T, DID, V, C>
// where
//     Self: Clone,
//     Ipld: Encode<C> + From<T>,
// {
//     fn encode<W: std::io::Write>(
//         &self,
//         codec: C,
//         w: &mut W,
//     ) -> Result<(), libipld_core::error::Error> {
//         let ipld: Ipld = self.clone().into();
//         ipld.encode(codec, w)
//     }
// }

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

// impl<
//         T: Verifiable<DID> + Capsule,
//         DID: Did,
//         V: varsig::Header<C>,
//         C: Codec + TryFrom<u32> + Into<u32>,
//     > Serialize for Envelope<T, DID, V, C>
// {
//     fn serialize<S>(&self, serializer: S) -> std::prelude::v1::Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         self.clone().serialize(serializer)
//     }
// }
//
// impl<
//         'de,
//         T: Verifiable<DID> + Capsule + TryFrom<Ipld>,
//         DID: Did,
//         V: varsig::Header<C>,
//         C: Codec + TryFrom<u32> + Into<u32>,
//     > Deserialize<'de> for Envelope<T, DID, V, C>
// where
//     Envelope<T, DID, V, C>: TryFrom<Ipld>,
//     <Envelope<T, DID, V, C> as TryFrom<Ipld>>::Error: std::fmt::Display,
// {
//     fn deserialize<D>(deserializer: D) -> std::prelude::v1::Result<Self, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         let ipld: Ipld = Deserialize::deserialize(deserializer)?;
//         Ok(Envelope::try_from(ipld).map_err(serde::de::Error::custom)?)
//     }
// }
