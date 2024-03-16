//! Policy language.
//!
//! The policy language is a simple predicate language extended with [`jq`]-style selectors.
//!
//! [`jq`]: https://stedolan.github.io/jq/

pub mod selector;

pub mod predicate;
pub use predicate::*;
