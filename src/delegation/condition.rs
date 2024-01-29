pub mod contains_all;
pub mod contains_any;
pub mod contains_key;
pub mod excludes_all;
pub mod excludes_key;
pub mod matches_regex;
pub mod max_length;
pub mod max_number;
pub mod min_length;
pub mod min_number;
pub mod traits;

use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde_derive::{Deserialize, Serialize};
use traits::Condition;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Common {
    ContainsAll(contains_all::ContainsAll),
    ContainsAny(contains_any::ContainsAny),
    ContainsKey(contains_key::ContainsKey),
    ExcludesKey(excludes_key::ExcludesKey),
    ExcludesAll(excludes_all::ExcludesAll),
    MinLength(min_length::MinLength),
    MaxLength(max_length::MaxLength),
    MinNumber(min_number::MinNumber),
    MaxNumber(max_number::MaxNumber),
    MatchesRegex(matches_regex::MatchesRegex),
}

impl From<Common> for Ipld {
    fn from(common: Common) -> Self {
        common.into()
    }
}

impl TryFrom<Ipld> for Common {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl Condition for Common {
    fn validate(&self, ipld: &Ipld) -> bool {
        match self {
            Common::ContainsAll(c) => c.validate(ipld),
            Common::ContainsAny(c) => c.validate(ipld),
            Common::ContainsKey(c) => c.validate(ipld),
            Common::ExcludesKey(c) => c.validate(ipld),
            Common::ExcludesAll(c) => c.validate(ipld),
            Common::MinLength(c) => c.validate(ipld),
            Common::MaxLength(c) => c.validate(ipld),
            Common::MinNumber(c) => c.validate(ipld),
            Common::MaxNumber(c) => c.validate(ipld),
            Common::MatchesRegex(c) => c.validate(ipld),
        }
    }
}
