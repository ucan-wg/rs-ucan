//! Cryptographic signature utilities

mod domain_separator;
mod nonce;

pub mod signature;
pub mod varsig;

pub use domain_separator::DomainSeparator;
pub use nonce::*;

#[cfg(feature = "bls")]
pub mod bls12381;

#[cfg(feature = "es512")]
pub mod es512;

#[cfg(feature = "rs256")]
pub mod rs256;

#[cfg(feature = "rs512")]
pub mod rs512;
