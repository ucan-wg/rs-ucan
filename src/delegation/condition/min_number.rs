//! A min number [`Condition`].
use super::traits::Condition;
use crate::{ability::arguments, ipld::Number};
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde_derive::{Deserialize, Serialize};

/// A minimum number [`Condition`]
///
/// A condition that checks if the length of an integer
/// or float is less than or equal to a set size.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MinNumber {
    /// Name of the field to check
    pub field: String,

    /// The minimum number
    pub min_number: Number,
}

impl From<MinNumber> for Ipld {
    fn from(min_number: MinNumber) -> Self {
        min_number.into()
    }
}

impl TryFrom<Ipld> for MinNumber {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl Condition for MinNumber {
    fn validate(&self, args: &arguments::Named<Ipld>) -> bool {
        match args.get(&self.field) {
            Some(Ipld::Integer(integer)) => match self.min_number {
                Number::Float(float) => *integer as f64 >= float,
                Number::Integer(integer) => integer >= integer,
            },
            Some(Ipld::Float(float)) => match self.min_number {
                Number::Float(float) => float >= float,
                Number::Integer(integer) => *float >= integer as f64, // FIXME this needs tests
            },
            _ => false,
        }
    }
}
