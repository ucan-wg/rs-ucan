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

use std::fmt::Debug;

// impl<A: Capability, Cond> DelegationPayload<A, Cond> {
//     fn check(
//         &self,
//         proof: &DelegationPayload<impl TryProven<A> + Capability, Cond>,
//         now: SystemTime,
//     ) -> Result<(), ()> {
//         // FIXME heavily WIP
//         // FIXME signature
//         self.check_time(now).unwrap();
//         self.check_issuer(&proof.audience)?; // FIXME alignment
//         self.check_subject(&proof.subject)?;
//         self.check_conditions(&proof.conditions)?;
//
//         proof.check_expiration(now)?;
//         proof.check_not_before(now)?;
//
//         self.check_ability(&proof.capability_builder)?;
//         Ok(())
//     }
// }

// FIXME Remove
// pub trait Prove<T> {
//     type Witness;
//     // FIXME make sure that passing the top-level item through and not checking each
//     // item in the chain against the next one is correct in the 1.0 semantics
//     fn prove<'a>(&'a self, proof: &'a T) -> &Self::Witness;
// }

// impl<T> Prove<DelegateAny> for T {
//     type Witness = T;
//
//     fn prove<'a>(&'a self, proof: &'a DelegateAny) -> &Self::Witness {
//         self
//     }
// }
//
// impl<T: Prove<T>> TryProve<T> for T {
//     type Error = Void;
//     type Proven = T;
//
//     fn try_prove<'a>(&'a self, proof: &'a T) -> Result<&'a T, Void> {
//         Ok(proof)
//     }
// }

// FIXME lives etirely in bindgen
// https://rustwasm.github.io/docs/wasm-bindgen/contributing/design/importing-js-struct.html
// pub struct DynamicJs {
//     pub command: Box<str>,
//     pub args: BTreeMap<Box<str>, Ipld>,
// }
//
// impl TryProve<DynamicJs> for DynamicJs {
//     type Error = ();
//     type Proven = DynamicJs;
//
//     fn try_prove<'a>(&'a self, candidate: &'a DynamicJs) -> Result<&'a DynamicJs, ()> {
//
//     }
// }

// impl ProveDelegaton<DynamicJs> for DynamicJs {
//     type Error = anyhow::Error;
//
//     fn prove(&self, proof: &DynamicJs) -> Result<Self, anyhow::Error> {
//         todo!()
//     }
// }
//

pub mod ability;
pub mod condition;
pub mod delegation;
pub mod invocation;
pub mod promise;
pub mod prove;
pub mod receipt;

//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////
//////////////////////////////////////

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
