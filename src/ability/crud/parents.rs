//! Flat types for parent checking.
//!
//! Types here turn recursive checking into a since union to check.
//! This only needs to handle "inner" delegation types, not the topmost `*`
//! ability, or the invocable leaves of a delegation hierarchy.

use super::error::ParentError;
use crate::proof::{parents::CheckParents, same::CheckSame};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg_attr(doc, aquamarine::aquamarine)]
/// The union of mutable parents.
///
/// This is helpful as a flat type to put in [`CheckParents::Parents`].
///
/// # Delegation Hierarchy
///
/// The parents captured here are highlted in the following diagram:
///
/// ```mermaid
/// flowchart TB
///     top("*")
///
///     subgraph CRUD Abilities
///       any("crud/*")
///
///       mutate("crud/mutate")
///
///       subgraph Invokable
///         read("crud/read")
///         create("crud/create")
///         update("crud/update")
///         destroy("crud/destroy")
///       end
///     end
///
///     readrun{{"invoke"}}
///     createrun{{"invoke"}}
///     updaterun{{"invoke"}}
///     destroyrun{{"invoke"}}
///
///     top --> any
///             any --> read -.-> readrun
///             any --> mutate
///                     mutate --> create -.-> createrun
///                     mutate --> update -.-> updaterun
///                     mutate --> destroy -.-> destroyrun
///
///     style any    stroke:orange;
///     style mutate stroke:orange;
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum MutableParents {
    /// The `crud/*` ability.
    Any(super::Any),

    /// The `crud/mutate` ability.
    Mutate(super::Mutate),
}

impl CheckSame for MutableParents {
    type Error = ParentError;

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        match self {
            MutableParents::Mutate(mutate) => match proof {
                MutableParents::Mutate(proof_mutate) => mutate
                    .check_same(proof_mutate)
                    .map_err(ParentError::InvalidMutateProof),

                MutableParents::Any(proof_any) => mutate
                    .check_parent(proof_any)
                    .map_err(ParentError::InvalidMutateParent),
            },

            MutableParents::Any(any) => match proof {
                MutableParents::Mutate(_) => Err(ParentError::CommandEscelation),
                MutableParents::Any(proof_any) => any
                    .check_same(proof_any)
                    .map_err(ParentError::InvalidAnyProof),
            },
        }
    }
}
