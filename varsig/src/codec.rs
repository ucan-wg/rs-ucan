//! IPLD Codec trait.

use alloc::vec::Vec;
use core::error::Error;

#[cfg(any(feature = "dag_cbor", feature = "dag_json"))]
use serde::{Deserialize, Serialize};

/// DAG-CBOR multicodec code.
#[cfg(feature = "dag_cbor")]
pub const DAG_CBOR_CODE: u64 = 0x71;

/// DAG-JSON multicodec code.
#[cfg(feature = "dag_json")]
pub const DAG_JSON_CODE: u64 = 0x0129;

/// DAG-CBOR codec marker type.
///
/// In `std` mode this is re-exported from `serde_ipld_dagcbor`.
/// In `no_std` mode this is a local unit struct with the same semantics.
#[cfg(all(feature = "dag_cbor", feature = "std"))]
pub use serde_ipld_dagcbor::codec::DagCborCodec;

/// DAG-CBOR codec marker type for `no_std` environments.
#[cfg(all(feature = "dag_cbor", not(feature = "std")))]
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DagCborCodec;

/// DAG-JSON codec marker type.
#[cfg(feature = "dag_json")]
pub use serde_ipld_dagjson::codec::DagJsonCodec;

/// IPLD Codec trait.
///
/// This trait is a generalization of the `libipld_core::codec::Codec` trait.
/// Specifically this allows an application to accept multiple codecs
/// and distinguish with a runtime enum. This is important for Varsig,
/// since it may need to encode to the configured codec for signature verification.
///
/// The API is slice-based (`Vec<u8>` / `&[u8]`) rather than stream-based
/// to support `no_std` environments.
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

    /// Encode the payload into a byte vector.
    ///
    /// # Errors
    ///
    /// If the encoding fails, it returns an error of type `Self::EncodingError`.
    fn encode_payload(&self, payload: &T) -> Result<Vec<u8>, Self::EncodingError>;

    /// Decode the payload from a byte slice.
    ///
    /// # Errors
    ///
    /// If the decoding fails, it returns an error of type `Self::DecodingError`.
    fn decode_payload(&self, bytes: &[u8]) -> Result<T, Self::DecodingError>;
}

// ---------------------------------------------------------------------------
// DAG-CBOR
// ---------------------------------------------------------------------------

/// DAG-CBOR encode error wrapper that implements [`core::error::Error`].
///
/// The upstream `serde_ipld_dagcbor` only implements `std::error::Error`,
/// which is unavailable in `no_std`. This newtype bridges the gap.
#[cfg(feature = "dag_cbor")]
#[derive(Debug)]
pub struct DagCborEncodeError(serde_ipld_dagcbor::EncodeError<alloc::collections::TryReserveError>);

#[cfg(feature = "dag_cbor")]
impl core::fmt::Display for DagCborEncodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[cfg(feature = "dag_cbor")]
impl Error for DagCborEncodeError {}

/// DAG-CBOR decode error wrapper that implements [`core::error::Error`].
#[cfg(feature = "dag_cbor")]
#[derive(Debug)]
pub struct DagCborDecodeError(serde_ipld_dagcbor::DecodeError<core::convert::Infallible>);

#[cfg(feature = "dag_cbor")]
impl core::fmt::Display for DagCborDecodeError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

#[cfg(feature = "dag_cbor")]
impl Error for DagCborDecodeError {}

#[cfg(all(feature = "dag_cbor", feature = "std"))]
impl<T: Serialize + for<'de> Deserialize<'de>> Codec<T> for DagCborCodec {
    type EncodingError = serde_ipld_dagcbor::error::CodecError;
    type DecodingError = serde_ipld_dagcbor::error::CodecError;

    fn multicodec_code(&self) -> u64 {
        DAG_CBOR_CODE
    }

    fn try_from_tags(code: &[u64]) -> Option<Self> {
        if code.len() != 1 {
            return None;
        }

        if *code.first()? == DAG_CBOR_CODE {
            Some(DagCborCodec)
        } else {
            None
        }
    }

    fn encode_payload(&self, payload: &T) -> Result<Vec<u8>, Self::EncodingError> {
        serde_ipld_dagcbor::to_vec(payload).map_err(serde_ipld_dagcbor::error::CodecError::from)
    }

    fn decode_payload(&self, bytes: &[u8]) -> Result<T, Self::DecodingError> {
        serde_ipld_dagcbor::from_slice(bytes).map_err(serde_ipld_dagcbor::error::CodecError::from)
    }
}

#[cfg(all(feature = "dag_cbor", not(feature = "std")))]
impl<T: Serialize + for<'de> Deserialize<'de>> Codec<T> for DagCborCodec {
    type EncodingError = DagCborEncodeError;
    type DecodingError = DagCborDecodeError;

    fn multicodec_code(&self) -> u64 {
        DAG_CBOR_CODE
    }

    fn try_from_tags(code: &[u64]) -> Option<Self> {
        if code.len() != 1 {
            return None;
        }

        if *code.first()? == DAG_CBOR_CODE {
            Some(DagCborCodec)
        } else {
            None
        }
    }

    fn encode_payload(&self, payload: &T) -> Result<Vec<u8>, Self::EncodingError> {
        serde_ipld_dagcbor::to_vec(payload).map_err(DagCborEncodeError)
    }

    fn decode_payload(&self, bytes: &[u8]) -> Result<T, Self::DecodingError> {
        serde_ipld_dagcbor::from_slice(bytes).map_err(DagCborDecodeError)
    }
}

// ---------------------------------------------------------------------------
// DAG-JSON
// ---------------------------------------------------------------------------

#[cfg(feature = "dag_json")]
impl<T: Serialize + for<'de> Deserialize<'de>> Codec<T> for DagJsonCodec {
    type EncodingError = serde_ipld_dagjson::error::CodecError;
    type DecodingError = serde_ipld_dagjson::error::CodecError;

    fn multicodec_code(&self) -> u64 {
        DAG_JSON_CODE
    }

    fn try_from_tags(code: &[u64]) -> Option<Self> {
        if code.len() != 1 {
            return None;
        }

        if code.first() == Some(&DAG_JSON_CODE) {
            Some(DagJsonCodec)
        } else {
            None
        }
    }

    fn encode_payload(&self, payload: &T) -> Result<Vec<u8>, Self::EncodingError> {
        serde_ipld_dagjson::to_vec(payload).map_err(serde_ipld_dagjson::error::CodecError::from)
    }

    fn decode_payload(&self, bytes: &[u8]) -> Result<T, Self::DecodingError> {
        serde_ipld_dagjson::from_slice(bytes).map_err(serde_ipld_dagjson::error::CodecError::from)
    }
}
