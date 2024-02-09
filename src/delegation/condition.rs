mod contains_all;
mod contains_any;
mod contains_key;
mod excludes_all;
mod excludes_key;
mod matches_regex;
mod max_length;
mod max_number;
mod min_length;
mod min_number;
mod traits;

pub use contains_all::ContainsAll;
pub use contains_any::ContainsAny;
pub use contains_key::ContainsKey;
pub use excludes_all::ExcludesAll;
pub use excludes_key::ExcludesKey;
pub use matches_regex::MatchesRegex;
pub use max_length::MaxLength;
pub use max_number::MaxNumber;
pub use min_length::MinLength;
pub use min_number::MinNumber;
pub use traits::Condition;

use crate::ability::arguments;
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde_derive::{Deserialize, Serialize};

/// The union of the common [`Condition`]s that ship directly with this library.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
#[allow(missing_docs)]
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
    fn validate(&self, args: &arguments::Named<Ipld>) -> bool {
        match self {
            Common::ContainsAll(c) => c.validate(args),
            Common::ContainsAny(c) => c.validate(args),
            Common::ContainsKey(c) => c.validate(args),
            Common::ExcludesKey(c) => c.validate(args),
            Common::ExcludesAll(c) => c.validate(args),
            Common::MinLength(c) => c.validate(args),
            Common::MaxLength(c) => c.validate(args),
            Common::MinNumber(c) => c.validate(args),
            Common::MaxNumber(c) => c.validate(args),
            Common::MatchesRegex(c) => c.validate(args),
        }
    }
}
