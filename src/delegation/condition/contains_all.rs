//! A [`Condition`] for ensuring a field contains all of a set of values.

use super::traits::Condition;
use crate::ability::arguments;
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde_derive::{Deserialize, Serialize};

/// A condition for ensuring a field contains all of a set of values.
///
/// This works on lists and maps. Maps will check the values, not keys.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContainsAll {
    /// Name of the field to check
    pub field: String,

    /// The elements that must be present
    pub contains_all: Vec<Ipld>,
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
    fn validate(&self, args: &arguments::Named) -> bool {
        match args.get(&self.field) {
            Some(Ipld::List(array)) => self.contains_all.iter().all(|ipld| array.contains(ipld)),
            Some(Ipld::Map(btree)) => {
                let vals: Vec<&Ipld> = btree.values().collect();
                self.contains_all.iter().all(|ipld| vals.contains(&ipld))
            }
            _ => false,
        }
    }
}
