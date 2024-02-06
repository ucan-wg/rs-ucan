use crate::did::Did;
use core::fmt;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

pub trait CheckSame {
    type Error;

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error>;
}

// Genereic
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Unequal;

// FIXME move under error.rs
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Error)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen(getter_with_clone))]
pub struct OptionalFieldErr {
    pub field: String,
    pub err: OptionalFieldReason,
}

impl fmt::Display for OptionalFieldErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Field {} is {}", self.field, self.err)
    }
}

// FIXME at minimum the name is confusing
#[derive(Copy, Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Error)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub enum OptionalFieldReason {
    MissingField,
    UnequalValue,
}

impl fmt::Display for OptionalFieldReason {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{}",
            match self {
                OptionalFieldReason::MissingField => "missing",
                OptionalFieldReason::UnequalValue => "unequal",
            }
        )
    }
}

impl CheckSame for Did {
    type Error = Unequal;

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        if self.eq(proof) {
            Ok(())
        } else {
            Err(Unequal)
        }
    }
}

impl<T: PartialEq> CheckSame for Option<T> {
    type Error = OptionalFieldReason;

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        match proof {
            None => Ok(()),
            Some(proof_) => match self {
                None => Err(OptionalFieldReason::MissingField),
                Some(self_) => {
                    if self_.eq(proof_) {
                        Ok(())
                    } else {
                        Err(OptionalFieldReason::UnequalValue)
                    }
                }
            },
        }
    }
}
