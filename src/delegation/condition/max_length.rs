use super::traits::Condition;
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaxLength {
    field: String,
    max_length: usize,
}

impl From<MaxLength> for Ipld {
    fn from(max_length: MaxLength) -> Self {
        max_length.into()
    }
}

impl TryFrom<Ipld> for MaxLength {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl Condition for MaxLength {
    fn validate(&self, ipld: &Ipld) -> bool {
        match ipld {
            Ipld::String(string) => string.len() <= self.max_length,
            Ipld::List(list) => list.len() <= self.max_length,
            Ipld::Map(map) => map.len() <= self.max_length,
            _ => false,
        }
    }
}
