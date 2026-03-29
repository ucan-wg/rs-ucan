//! Core UCAN functionality.

#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::multiple_crate_versions)] // syn
#![cfg_attr(docsrs, feature(doc_cfg))]

extern crate alloc;

pub mod cid;
pub mod collection;
pub mod collections;
pub mod command;
pub mod crypto;
pub mod delegation;
pub mod did;
pub mod envelope;
pub mod invocation;
pub mod number;
pub mod promise;
// pub mod receipt; TODO Reenable after first release
pub mod task;
pub mod time;
pub mod unset;

// Internal modules
mod ipld;
mod sealed;

pub use delegation::{builder::DelegationBuilder, Delegation};
// pub use invocation::{builder::InvocationBuilder, Invocation};
