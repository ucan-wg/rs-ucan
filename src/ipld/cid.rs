//! Utilities for [`Cid`]s

use crate::ipld;
use libipld_core::{cid::Cid, ipld::Ipld};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_derive::TryFromJsValue;

#[cfg(feature = "test_utils")]
use proptest::prelude::*;

#[cfg(feature = "test_utils")]
use crate::test_utils::SomeCodec;

#[cfg(feature = "test_utils")]
use crate::test_utils::SomeMultihash;

/// A newtype wrapper around a [`Cid`]
///
/// This is largely to attach traits to [`Cid`]s, such as [`wasm_bindgen`] conversions.
#[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
pub struct Newtype {
    pub cid: Cid,
}

/// A newtype wrapper around a [`Cid`]
///
/// This is largely to attach traits to [`Cid`]s, such as [`wasm_bindgen`] conversions.
#[cfg(target_arch = "wasm32")]
#[derive(Debug, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[wasm_bindgen]
pub struct Newtype {
    #[wasm_bindgen(skip)]
    pub cid: Cid,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    /// This is here because the TryFromJsValue derivation macro
    /// doesn't automatically support `Option<T>`.
    ///
    /// [https://docs.rs/wasm-bindgen-derive/0.2.1/wasm_bindgen_derive/#optional-arguments]
    #[wasm_bindgen(typescript_type = "Newtype | undefined")]
    pub type OptionNewtype;
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl Newtype {
    /// Parse a [`Newtype`] from a string
    pub fn from_string(cid_string: String) -> Result<Newtype, JsError> {
        Newtype::try_from(cid_string).map_err(|e| JsError::new(&format!("{}", e)))
    }

    pub fn try_from_js_value(js: &JsValue) -> Result<Newtype, JsError> {
        match &js.as_string() {
            Some(s) => Newtype::from_string(s.clone()),
            None => Err(JsError::new("Expected a string")),
        }
    }
}

impl Newtype {
    /// Convert the [`Cid`] to a string
    pub fn to_string(&self) -> String {
        self.cid.to_string()
    }
}

impl TryFrom<String> for Newtype {
    type Error = <Cid as TryFrom<String>>::Error;

    fn try_from(cid_string: String) -> Result<Self, Self::Error> {
        Cid::try_from(cid_string).map(Into::into)
    }
}

impl From<Newtype> for Cid {
    fn from(wrapper: Newtype) -> Self {
        wrapper.cid
    }
}

impl From<Cid> for Newtype {
    fn from(cid: Cid) -> Self {
        Self { cid }
    }
}

impl TryFrom<Ipld> for Newtype {
    type Error = NotACid;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::Link(cid) => Ok(Newtype { cid }),
            other => Err(NotACid(other.into())),
        }
    }
}

impl TryFrom<&Ipld> for Newtype {
    type Error = NotACid;

    fn try_from(ipld: &Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::Link(cid) => Ok(Newtype { cid: *cid }),
            other => Err(NotACid(other.clone().into())),
        }
    }
}

// #[cfg(not(target_arch = "wasm32"))]
#[derive(Debug, PartialEq, Clone, Error, Serialize, Deserialize)]
#[error("Not a CID: {0:?}")]
pub struct NotACid(pub ipld::Newtype);

#[cfg(feature = "test_utils")]
impl Arbitrary for Newtype {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        // Very much faking it
        any::<([u8; 32], SomeMultihash, SomeCodec)>()
            .prop_map(|(hash_bytes, hasher, codec)| {
                let multihash = MultihashGeneric::wrap(hasher.0.into(), &hash_bytes.as_slice())
                    .expect("Sha2_256 should always successfully encode a hash");

                let cid = Cid::new_v1(codec.0.into(), multihash);
                Newtype { cid }
            })
            .boxed()
    }
}
