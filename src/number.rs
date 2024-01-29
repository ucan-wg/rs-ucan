use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Number {
    Float(f64),
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
