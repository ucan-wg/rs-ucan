//! A max length [`Condition`].
use super::traits::Condition;
use crate::ability::arguments;
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde_derive::{Deserialize, Serialize};

/// A maximum length [`Condition`]
///
/// A condition that checks if the length of a string, list,
/// or map is less than or equal to a set size.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaxLength {
    /// Name of the field to check
    pub field: String,

    /// The maximum length
    pub max_length: usize,
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
    fn validate(&self, args: &arguments::Named) -> bool {
        match args.get(&self.field) {
            Some(Ipld::String(string)) => string.len() <= self.max_length,
            Some(Ipld::List(list)) => list.len() <= self.max_length,
            Some(Ipld::Map(map)) => map.len() <= self.max_length,
            _ => false,
        }
    }
}
