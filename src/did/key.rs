//! Support for the [`did:key`](https://w3c-ccg.github.io/did-method-key/) DID method.

mod signature;
mod verifier;

pub mod traits;

pub use signature::Signature;
pub use verifier::*;
