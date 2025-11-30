//! Core UCAN functionality.

#![allow(clippy::multiple_crate_versions)] // syn
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod cid;
pub mod collection;
pub mod command;
pub mod crypto;
pub mod delegation;
pub mod did;
pub mod envelope;
pub mod future;
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
