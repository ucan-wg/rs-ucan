//! Cryptographic signature utilities

pub mod domain_separator;

#[cfg(feature = "bls")]
pub mod bls12381;

#[cfg(feature = "es512")]
pub mod p521;

#[cfg(feature = "eddsa")]
pub mod eddsa;

#[cfg(feature = "es256")]
pub mod es256;

#[cfg(feature = "es256k")]
pub mod es256k;

#[cfg(feature = "es384")]
pub mod es384;

#[cfg(feature = "es512")]
pub mod es512;

#[cfg(feature = "ps256")]
pub mod ps256;

#[cfg(feature = "rs256")]
pub mod rs256;

#[cfg(feature = "rs512")]
pub mod rs512;
