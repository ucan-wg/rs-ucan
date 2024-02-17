//! Decentralized Identifier ([DID][wiki]) utilities.
//!
//! [wiki]: https://en.wikipedia.org/wiki/Decentralized_identifier

mod newtype;
mod traits;

pub mod key;
pub mod preset;

pub use newtype::{FromIpldError, Newtype};
pub use traits::{Did, Verifiable};
