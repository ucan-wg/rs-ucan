//! CRUD-specific errors

use crate::{
    ability::arguments,
    proof::{error::OptionalFieldError, parents::CheckParents, same::CheckSame},
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Error, Serialize, Deserialize)]
pub enum ProofError {
    #[error("An issue with the path field")]
    Path(#[from] OptionalFieldError),

    #[error("An issue with the (inner) arguments field")]
    Args(#[from] arguments::NamedError),

    #[error("Proof has `args`, but none were present on delegate")]
    MissingProofArgs,
}

/// Error cases when checking [`MutableParents`] proofs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Error)]
pub enum ParentError {
    /// Error when comparing `crud/*` to another `crud/*`.
    #[error(transparent)]
    InvalidAnyProof(<super::Any as CheckSame>::Error),

    /// Error when comparing `crud/mutate` to another `crud/mutate`.
    #[error(transparent)]
    InvalidMutateProof(<super::Mutate as CheckSame>::Error),

    /// Error when comparing `crud/*` as a proof for `crud/mutate`.
    #[error(transparent)]
    InvalidMutateParent(<super::Mutate as CheckParents>::ParentError),

    /// "Expected `crud/*`, but got `crud/mutate`".
    #[error("Expected `crud/*`, but got `crud/mutate`")]
    CommandEscelation,
}
