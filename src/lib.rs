#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_debug_implementations, missing_docs, rust_2018_idioms)]
#![deny(unreachable_pub)]

//! rs-ucan

use std::str::FromStr;

use cid::{multihash, Cid};
use serde::{de, Deserialize, Deserializer, Serialize};

pub mod builder;
pub mod capability;
pub mod crypto;
pub mod did_verifier;
pub mod error;
pub mod plugins;
pub mod semantics;
pub mod store;
pub mod time;
pub mod ucan;

#[cfg(target_arch = "wasm32")]
mod wasm;

#[cfg(target_arch = "wasm32")]
pub use wasm::*;

#[doc(hidden)]
#[cfg(not(target_arch = "wasm32"))]
pub use linkme;

/// The default multihash algorithm used for UCANs
pub const DEFAULT_MULTIHASH: multihash::Code = multihash::Code::Sha2_256;

/// A decentralized identifier.
pub type Did = String;

/// The empty fact
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmptyFact {}

/// The default fact
pub type DefaultFact = EmptyFact;

/// A newtype around Cid that (de)serializes as a string
#[derive(Debug, Clone)]
pub struct CidString(pub(crate) Cid);

impl Serialize for CidString {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.0.to_string().as_str())
    }
}

impl<'de> Deserialize<'de> for CidString {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;

        Cid::from_str(&s)
            .map(CidString)
            .map_err(|e| de::Error::custom(format!("invalid CID: {}", e)))
    }
}

/// Test utilities.
#[cfg(any(test, feature = "test_utils"))]
#[cfg_attr(docsrs, doc(cfg(feature = "test_utils")))]
pub mod test_utils;
