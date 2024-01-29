use super::traits::Condition;
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContainsAll {
    field: String,
    contains_all: Vec<Ipld>,
}

impl From<ContainsAll> for Ipld {
    fn from(contains_all: ContainsAll) -> Self {
        contains_all.into()
    }
}

impl TryFrom<Ipld> for ContainsAll {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl Condition for ContainsAll {
    fn validate(&self, ipld: &Ipld) -> bool {
        match ipld {
            Ipld::List(array) => self.contains_all.iter().all(|ipld| array.contains(ipld)),
            Ipld::Map(btree) => {
                let vals: Vec<&Ipld> = btree.values().collect();
                self.contains_all.iter().all(|ipld| vals.contains(&ipld))
            }
            _ => false,
        }
    }
}
