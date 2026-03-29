//! IPLD Codec trait.

use std::{
    error::Error,
    io::{BufRead, Write},
};

use ipld_core::codec::Codec as IpldCodec;
use serde::{Deserialize, Serialize};
use serde_ipld_dagcbor::{codec::DagCborCodec, error::CodecError};

#[cfg(feature = "dag_json")]
use serde_ipld_dagjson::{codec::DagJsonCodec, error::CodecError as JsonError};

/// IPLD Codec trait.
///
/// This trait is a generalization of the `libipld_core::codec::Codec` trait.
/// Specifically this allows an application to accept multiple codecs
/// and distinguish with a runtime enum. This is important for Varsig,
/// since it may need to encode to the configured codec for signature verification.
///
/// An implementation is provided for types that have `ipld_core::codec::Codec`.
pub trait Codec<T>: Sized {
    /// Encoding error type.
    type EncodingError: Error;

    /// Decoding error type.
    type DecodingError: Error;

    /// Multicodec code.
    ///
    /// This is not a `const` because an implementation may
    /// support more than one IPLD codec, so it is runtime dependent.
    fn multicodec_code(&self) -> u64;

    /// Try to create a codec from a series of tags.
    fn try_from_tags(code: &[u64]) -> Option<Self>;

    /// Encode the payload to the given buffer.
    ///
    /// ## Parameters
    ///
    /// - `payload`: The payload to encode.
    /// - `buffer`: The buffer to write the encoded payload to.
    ///
    /// ## Returns
    ///
    /// Returns `Ok(())` on success, or an error of type `Self::EncodingError` on failure.
    ///
    /// ## Errors
    ///
    /// If the encoding fails, it returns an error of type `Self::EncodingError`.
    fn encode_payload<W: Write>(
        &self,
        payload: &T,
        buffer: &mut W,
    ) -> Result<(), Self::EncodingError>;

    /// Decode the payload from the given reader.
    ///
    /// ## Parameters
    ///
    /// - `reader`: The reader to read the encoded payload from.
    ///
    /// ## Returns
    ///
    /// Returns the decoded payload of type `T` on success,
    /// or an error of type `Self::DecodingError` on failure.
    ///
    /// ## Errors
    ///
    /// If the decoding fails, it returns an error of type `Self::DecodingError`.
    fn decode_payload<R: BufRead>(&self, reader: &mut R) -> Result<T, Self::DecodingError>;
}

impl<T: Serialize + for<'de> Deserialize<'de>> Codec<T> for DagCborCodec {
    type EncodingError = CodecError;
    type DecodingError = CodecError;

    fn multicodec_code(&self) -> u64 {
        <DagCborCodec as IpldCodec<T>>::CODE
    }

    fn try_from_tags(code: &[u64]) -> Option<Self> {
        if code.is_empty() {
            return None;
        }

        if code.len() > 1 {
            return None;
        }

        if *code.first()? == <DagCborCodec as IpldCodec<T>>::CODE {
            Some(DagCborCodec)
        } else {
            None
        }
    }

    fn encode_payload<W: Write>(
        &self,
        payload: &T,
        buffer: &mut W,
    ) -> Result<(), Self::EncodingError> {
        DagCborCodec::encode(buffer, payload)
    }

    fn decode_payload<R: BufRead>(&self, reader: &mut R) -> Result<T, Self::DecodingError> {
        DagCborCodec::decode(reader)
    }
}

#[cfg(feature = "dag_json")]
impl<T: Serialize + for<'de> Deserialize<'de>> Codec<T> for DagJsonCodec {
    type EncodingError = JsonError;
    type DecodingError = JsonError;

    fn multicodec_code(&self) -> u64 {
        <DagJsonCodec as IpldCodec<T>>::CODE
    }

    fn try_from_tags(code: &[u64]) -> Option<Self> {
        if code.is_empty() {
            return None;
        }

        if code.len() > 1 {
            return None;
        }

        if code.first() == Some(&<DagJsonCodec as IpldCodec<T>>::CODE) {
            Some(DagJsonCodec)
        } else {
            None
        }
    }

    fn encode_payload<W: Write>(
        &self,
        payload: &T,
        buffer: &mut W,
    ) -> Result<(), Self::EncodingError> {
        DagJsonCodec::encode(buffer, payload)
    }

    fn decode_payload<R: BufRead>(&self, reader: &mut R) -> Result<T, Self::DecodingError> {
        DagJsonCodec::decode(reader)
    }
}
