//! A [`Condition`] for ensuring a field contains some of a set of values.
use super::traits::Condition;
use crate::ability::arguments;
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde_derive::{Deserialize, Serialize};

/// A [`Condition`] for ensuring a field contains one or more of a set of values.
///
/// This works on lists and maps. Maps will check the values, not keys.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContainsAny {
    /// Name of the field to check.
    pub field: String,

    /// The elements that must be present.
    pub contains_any: Vec<Ipld>,
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
    fn validate(&self, args: &arguments::Named) -> bool {
        match args.get(&self.field) {
            Some(Ipld::List(array)) => array.iter().any(|ipld| self.contains_any.contains(ipld)),
            Some(Ipld::Map(btree)) => {
                let vals: Vec<&Ipld> = btree.values().collect();
                self.contains_any.iter().any(|ipld| vals.contains(&ipld))
            }
            _ => false,
        }
    }
}
