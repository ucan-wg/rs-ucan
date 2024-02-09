//! A min length [`Condition`].
use super::traits::Condition;
use crate::ability::arguments;
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde_derive::{Deserialize, Serialize};

/// A mimimum length [`Condition`]
///
/// This checks if the length of a string, list,
/// or map is greater than or equal to a set size.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MinLength {
    /// Name of the field to check
    pub field: String,

    /// The minimum length
    pub min_length: usize,
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
    fn validate(&self, args: &arguments::Named<Ipld>) -> bool {
        match args.get(&self.field) {
            Some(Ipld::String(string)) => string.len() >= self.min_length,
            Some(Ipld::List(list)) => list.len() >= self.min_length,
            Some(Ipld::Map(map)) => map.len() >= self.min_length,
            _ => false,
        }
    }
}
