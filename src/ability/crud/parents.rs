//! Flat types for parent checking.
//!
//! Types here turn recursive checking into a since union to check.
//! This only needs to handle "inner" delegation types, not the topmost `*`
//! ability, or the invocable leaves of a delegation hierarchy.

use crate::proof::{parents::CheckParents, same::CheckSame};
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// The union of mutable parents.
///
/// This is helpful as a flat type to put in [`CheckParents::Parents`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum MutableParents {
    /// The `crud/*` ability.
    Any(super::Any),

    /// The `crud/mutate` ability.
    Mutate(super::Mutate),
}

impl CheckSame for MutableParents {
    type Error = Error;

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        match self {
            MutableParents::Mutate(mutate) => match proof {
                MutableParents::Mutate(proof_mutate) => mutate
                    .check_same(proof_mutate)
                    .map_err(Error::InvalidMutateProof),

                MutableParents::Any(proof_any) => mutate
                    .check_parent(proof_any)
                    .map_err(Error::InvalidMutateParent),
            },

            MutableParents::Any(any) => match proof {
                MutableParents::Mutate(_) => Err(Error::CommandEscelation),
                MutableParents::Any(proof_any) => {
                    any.check_same(proof_any).map_err(Error::InvalidAnyProof)
                }
            },
        }
    }
}

/// Error cases when checking [`MutableParents`] proofs
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Error)]
pub enum Error {
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
