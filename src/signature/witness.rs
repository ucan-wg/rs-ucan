//! Signatures and related cryptographic witnesses.

use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use signature::SignatureEncoding;

/// Asymmetric cryptographic witnesses.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Witness<S> {
    /// A single cryptographic signature.
    Signature(S),
    // FIXME TODO
    // Batch {
    //     signature: S,
    //     root: Vec<u8>,
    //     merkle_proof: Vec<MerkleStep>,
    // },
}

impl<S: SignatureEncoding> From<Witness<S>> for Ipld {
    fn from(w: Witness<S>) -> Self {
        match w {
            Witness::Signature(sig) => sig.to_vec().into(),
        }
    }
}
