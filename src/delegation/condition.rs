//! Conditions for syntactic validation of abilities in [`Delegation`][super::Delegation]s.

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
pub enum Preset {
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

impl From<Preset> for Ipld {
    fn from(common: Preset) -> Self {
        common.into()
    }
}

impl TryFrom<Ipld> for Preset {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl Condition for Preset {
    fn validate(&self, args: &arguments::Named<Ipld>) -> bool {
        match self {
            Preset::ContainsAll(c) => c.validate(args),
            Preset::ContainsAny(c) => c.validate(args),
            Preset::ContainsKey(c) => c.validate(args),
            Preset::ExcludesKey(c) => c.validate(args),
            Preset::ExcludesAll(c) => c.validate(args),
            Preset::MinLength(c) => c.validate(args),
            Preset::MaxLength(c) => c.validate(args),
            Preset::MinNumber(c) => c.validate(args),
            Preset::MaxNumber(c) => c.validate(args),
            Preset::MatchesRegex(c) => c.validate(args),
        }
    }
}
