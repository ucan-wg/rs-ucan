use super::traits::Condition;
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExcludesKey {
    field: String,
    excludes_key: String,
}

impl From<ExcludesKey> for Ipld {
    fn from(excludes_key: ExcludesKey) -> Self {
        excludes_key.into()
    }
}

impl TryFrom<Ipld> for ExcludesKey {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl Condition for ExcludesKey {
    fn validate(&self, ipld: &Ipld) -> bool {
        match ipld {
            Ipld::Map(map) => map.get(&self.field).is_none(),
            _ => false,
        }
    }
}
