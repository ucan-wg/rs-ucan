use super::traits::Condition;
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExcludesAll {
    field: String,
    excludes_all: Vec<Ipld>,
}

impl From<ExcludesAll> for Ipld {
    fn from(excludes_all: ExcludesAll) -> Self {
        excludes_all.into()
    }
}

impl TryFrom<Ipld> for ExcludesAll {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl Condition for ExcludesAll {
    fn validate(&self, ipld: &Ipld) -> bool {
        match ipld {
            Ipld::List(array) => self.excludes_all.iter().all(|ipld| !array.contains(ipld)),
            Ipld::Map(btree) => {
                let vals: Vec<&Ipld> = btree.values().collect();
                self.excludes_all.iter().all(|ipld| !vals.contains(&ipld))
            }
            _ => false,
        }
    }
}
