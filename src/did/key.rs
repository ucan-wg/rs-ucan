//! Support for the [`did:key`](https://w3c-ccg.github.io/did-method-key/) DID method.

mod signature;
mod verifier;
mod signer;

pub mod traits;

pub use signature::Signature;
pub use verifier::*;
pub use signer::*;
