use super::traits::Condition;
use crate::number::Number;
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaxNumber {
    field: String,
    max_number: Number,
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
    fn validate(&self, ipld: &Ipld) -> bool {
        match ipld {
            Ipld::Integer(integer) => match self.max_number {
                Number::Float(float) => *integer as f64 <= float,
                Number::Integer(integer) => integer <= integer,
            },
            Ipld::Float(float) => match self.max_number {
                Number::Float(float) => float <= float,
                Number::Integer(integer) => *float <= integer as f64, // FIXME this needs tests
            },
            _ => false,
        }
    }
}
