use super::traits::Condition;
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContainsAny {
    field: String,
    contains_any: Vec<Ipld>,
}

impl From<ContainsAny> for Ipld {
    fn from(contains_any: ContainsAny) -> Self {
        contains_any.into()
    }
}

impl TryFrom<Ipld> for ContainsAny {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl Condition for ContainsAny {
    fn validate(&self, ipld: &Ipld) -> bool {
        match ipld {
            Ipld::List(array) => array.iter().any(|ipld| self.contains_any.contains(ipld)),
            Ipld::Map(btree) => {
                let vals: Vec<&Ipld> = btree.values().collect();
                self.contains_any.iter().any(|ipld| vals.contains(&ipld))
            }
            _ => false,
        }
    }
}
