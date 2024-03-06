//! Helpers for working with [`Ipld`][libipld_core::ipld::Ipld].
//!
//! [`Ipld`] is a fully concrete data type, and only has a few trait implementations.
//! This module provides a few newtype wrappers that allow you to add trait implementations,
//! and generalized forms to embed non-IPLD into IPLD structure.
//!
//! [`Ipld`]: libipld_core::ipld::Ipld

mod collection;
mod number;
mod promised;

pub mod cid;
pub mod newtype;

pub use collection::Collection;
pub use newtype::Newtype;
pub use number::Number;
pub use promised::*;
