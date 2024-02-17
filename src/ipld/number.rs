//! Helpers for working with [`Ipld`] numerics.

use enum_as_inner::EnumAsInner;
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde_derive::{Deserialize, Serialize};

/// The union of [`Ipld`] numeric types
///
/// This is helpful when comparing different numeric types, such as
/// bounds checking in [`Condition`]s.
///
/// [`Condition`]: crate::delegation::Condition
#[derive(Debug, Clone, PartialEq, PartialOrd, EnumAsInner, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Number {
    /// Designate a floating point number
    Float(f64),

    /// Designate an integer
    Integer(i128),
}

impl From<Number> for Ipld {
    fn from(number: Number) -> Self {
        number.into()
    }
}

impl TryFrom<Ipld> for Number {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl From<i128> for Number {
    fn from(i: i128) -> Number {
        Number::Integer(i)
    }
}

impl From<f64> for Number {
    fn from(f: f64) -> Number {
        Number::Float(f)
    }
}
