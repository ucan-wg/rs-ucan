use super::traits::Condition;
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MinLength {
    field: String,
    min_length: usize,
}

impl From<MinLength> for Ipld {
    fn from(min_length: MinLength) -> Self {
        min_length.into()
    }
}

impl TryFrom<Ipld> for MinLength {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl Condition for MinLength {
    fn validate(&self, ipld: &Ipld) -> bool {
        match ipld {
            Ipld::String(string) => string.len() >= self.min_length,
            Ipld::List(list) => list.len() >= self.min_length,
            Ipld::Map(map) => map.len() >= self.min_length,
            _ => false,
        }
    }
}
