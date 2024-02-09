use crate::ability::arguments;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, PartialEq, Error, Serialize, Deserialize)]
pub enum ProofError {
    #[error("An issue with the path field")]
    Path(#[from] PathError),

    #[error("An issue with the (inner) arguments field")]
    Args(#[from] arguments::NamedError),

    #[error("Proof `args` were expected, but none were present")]
    MissingProofArgs,
}

#[derive(Debug, Clone, PartialEq, Error, Serialize, Deserialize)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub enum PathError {
    #[error("Path required in proof, but was not present")]
    Missing,

    #[error("Proof path did not match")]
    Mismatch,
}
