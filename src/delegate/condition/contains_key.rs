use super::traits::Condition;
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContainsKey {
    field: String,
    contains_key: String,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    with_value: Option<Ipld>,
}

impl From<ContainsKey> for Ipld {
    fn from(contains_key: ContainsKey) -> Self {
        contains_key.into()
    }
}

impl TryFrom<Ipld> for ContainsKey {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl Condition for ContainsKey {
    fn validate(&self, ipld: &Ipld) -> bool {
        match ipld {
            Ipld::Map(map) => {
                if let Some(value) = map.get(&self.field) {
                    if let Some(with_value) = &self.with_value {
                        value == with_value
                    } else {
                        true
                    }
                } else {
                    false
                }
            }
            _ => false,
        }
    }
}
