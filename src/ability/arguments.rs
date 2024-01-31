use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Arguments(pub BTreeMap<String, Ipld>);

impl Arguments {
    pub fn from_iter(iterable: impl IntoIterator<Item = (String, Ipld)>) -> Self {
        Arguments(iterable.into_iter().collect())
    }
}

impl TryFrom<Ipld> for Arguments {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}
