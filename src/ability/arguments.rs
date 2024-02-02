use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Arguments(pub BTreeMap<String, Ipld>);

impl Arguments {
    pub fn from_iter(iterable: impl IntoIterator<Item = (String, Ipld)>) -> Self {
        Arguments(iterable.into_iter().collect())
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Ipld)> {
        self.0.iter()
    }

    pub fn into_iter(self) -> impl Iterator<Item = (String, Ipld)> {
        self.0.into_iter()
    }
}

impl Arguments {
    pub fn insert(&mut self, key: String, value: Ipld) -> Option<Ipld> {
        self.0.insert(key, value)
    }

    pub fn get(&self, key: &str) -> Option<&Ipld> {
        self.0.get(key)
    }
}

impl Default for Arguments {
    fn default() -> Self {
        Arguments(BTreeMap::new())
    }
}

impl TryFrom<Ipld> for Arguments {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}
