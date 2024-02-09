//! A max number [`Condition`].
use super::traits::Condition;
use crate::{ability::arguments, number::Number};
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde_derive::{Deserialize, Serialize};

/// A maximum number [`Condition`]
///
/// A condition that checks if the length of an integer
/// or float is less than or equal to a set size.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaxNumber {
    /// Name of the field to check
    pub field: String,

    /// The maximum number
    pub max_number: Number,
}

impl From<MaxNumber> for Ipld {
    fn from(max_number: MaxNumber) -> Self {
        max_number.into()
    }
}

impl TryFrom<Ipld> for MaxNumber {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl Condition for MaxNumber {
    fn validate(&self, args: &arguments::Named<Ipld>) -> bool {
        match args.get(&self.field) {
            Some(Ipld::Integer(integer)) => match self.max_number {
                Number::Float(float) => *integer as f64 <= float,
                Number::Integer(integer) => integer <= integer,
            },
            Some(Ipld::Float(float)) => match self.max_number {
                Number::Float(float) => float <= float,
                Number::Integer(integer) => *float <= integer as f64, // FIXME this needs tests
            },
            _ => false,
        }
    }
}
