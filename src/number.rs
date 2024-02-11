//! Helpers for working with [`Ipld`] numerics.

use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde_derive::{Deserialize, Serialize};

/// The union of [`Ipld`] numeric types
#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
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
