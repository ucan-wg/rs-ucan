//! Standatd error types for delegation checking.

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

/// An error for when values are unequal.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize, Error)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
#[error("unequal")]
pub struct Unequal();

/// A generic error for when two fields are unequal.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Error)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub enum OptionalFieldError {
    /// A required field is missing.
    ///
    /// For example, when its proof has a vaue, but the target does not.
    #[error("Field missing")]
    Missing,

    /// A field is present but has a different value in its proof
    #[error("Field value unequal")]
    Unequal,
}

impl From<Unequal> for OptionalFieldError {
    fn from(_: Unequal) -> Self {
        OptionalFieldError::Unequal
    }
}

impl TryFrom<OptionalFieldError> for Unequal {
    type Error = OptionalFieldError;

    fn try_from(e: OptionalFieldError) -> Result<Self, Self::Error> {
        match e {
            OptionalFieldError::Unequal => Ok(Unequal {}),
            _ => Err(e),
        }
    }
}
