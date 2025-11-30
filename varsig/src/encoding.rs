//! Preset IPLD encoding types.

use crate::codec::Codec;
use serde::{Deserialize, Serialize};
use std::io::{BufRead, Write};
use thiserror::Error;

/// IPLD encoding types.
#[repr(u64)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Encoding {
    /// `DAG-CBOR` encoding.
    #[cfg(feature = "dag_cbor")]
    DagCbor = 0x71,

    /// `DAG-JSON` encoding.
    #[cfg(feature = "dag_json")]
    DagJson = 0x0129,

    /// Canonicalized JWT encoding.
    #[cfg(feature = "jwt")]
    Jwt = 0x6a77,

    /// EIP-191 encoding.
    #[cfg(feature = "eip191")]
    Eip191 = 0xe191,
}

impl<T: Serialize + for<'a> Deserialize<'a>> Codec<T> for Encoding {
    type EncodingError = EncodingError;
    type DecodingError = DecodingError;

    fn multicodec_code(&self) -> u64 {
        *self as u64
    }

    fn try_from_tags(code: &[u64]) -> Option<Self> {
        if code.is_empty() {
            return None;
        }

        if code.len() > 1 {
            return None;
        }

        match code.first()? {
            #[cfg(feature = "dag_cbor")]
            0x71 => Some(Encoding::DagCbor),

            #[cfg(feature = "dag_json")]
            0x0129 => Some(Encoding::DagJson),

            #[cfg(feature = "jwt")]
            0x6a77 => Some(Encoding::Jwt),

            #[cfg(feature = "eip191")]
            0xe191 => Some(Encoding::Eip191),

            _ => None,
        }
    }

    /// Encode the payload to the given buffer.
    fn encode_payload<W: Write>(
        &self,
        payload: &T,
        buffer: &mut W,
    ) -> Result<(), Self::EncodingError> {
        match self {
            #[cfg(feature = "dag_cbor")]
            Encoding::DagCbor => Ok(serde_ipld_dagcbor::to_writer(buffer, payload)?),

            #[cfg(feature = "dag_json")]
            Encoding::DagJson => Ok(serde_ipld_dagjson::to_writer(buffer, payload)?),

            #[cfg(feature = "jwt")]
            Encoding::Jwt => todo!(),

            #[cfg(feature = "eip191")]
            Encoding::Eip191 => todo!(),
        }
    }

    /// Decode the payload from the given reader buffer.
    fn decode_payload<R: BufRead>(&self, reader: &mut R) -> Result<T, Self::DecodingError> {
        match self {
            #[cfg(feature = "dag_cbor")]
            Encoding::DagCbor => Ok(serde_ipld_dagcbor::from_reader(reader)?),

            #[cfg(feature = "dag_json")]
            Encoding::DagJson => Ok(serde_ipld_dagjson::from_reader(reader)?),

            #[cfg(feature = "jwt")]
            Encoding::Jwt => todo!(),

            #[cfg(feature = "eip191")]
            Encoding::Eip191 => todo!(),
        }
    }
}

/// Encoding errors for the enabled encoding types.
#[derive(Debug, Error)]
pub enum EncodingError {
    /// Encoding error from `DAG-CBOR`.
    #[cfg(feature = "dag_cbor")]
    #[error(transparent)]
    CborError(#[from] serde_ipld_dagcbor::EncodeError<std::io::Error>),

    /// Encoding error from `DAG-JSON`.
    #[cfg(feature = "dag_json")]
    #[error(transparent)]
    JsonError(#[from] serde_ipld_dagjson::error::EncodeError),
}

/// Decoding errors for the enabled encoding types.
#[derive(Debug, Error)]
pub enum DecodingError {
    /// Decoding error from `DAG-CBOR`.
    #[cfg(feature = "dag_cbor")]
    #[error(transparent)]
    CborError(#[from] serde_ipld_dagcbor::DecodeError<std::io::Error>),

    /// Decoding error from `DAG-JSON`.
    #[cfg(feature = "dag_json")]
    #[error(transparent)]
    JsonError(#[from] serde_ipld_dagjson::error::DecodeError),
}
