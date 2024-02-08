//! A regular expression [`Condition`].
use super::traits::Condition;
use crate::ability::arguments;
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use regex::Regex;
use serde_derive::{Deserialize, Serialize};

/// A regular expression [`Condition`]
///
/// This checks a string against a regular expression.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MatchesRegex {
    /// Name of the field to check
    pub field: String,

    /// The minimum length
    pub matches_regex: Matcher,
}

impl From<MatchesRegex> for Ipld {
    fn from(matches_regex: MatchesRegex) -> Self {
        matches_regex.into()
    }
}

impl TryFrom<Ipld> for MatchesRegex {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl Condition for MatchesRegex {
    fn validate(&self, args: &arguments::Named) -> bool {
        match args.get(&self.field) {
            Some(Ipld::String(string)) => self.matches_regex.0.is_match(string),
            _ => false,
        }
    }
}

/// A newtype wrapper around [`Regex`]
#[derive(Debug, Clone)]
pub struct Matcher(Regex);

impl PartialEq for Matcher {
    fn eq(&self, other: &Self) -> bool {
        self.0.as_str() == other.0.as_str()
    }
}

impl Eq for Matcher {}

impl serde::Serialize for Matcher {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.0.as_str().serialize(serializer)
    }
}

impl<'de> serde::Deserialize<'de> for Matcher {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s: &str = serde::Deserialize::deserialize(deserializer)?;
        match Regex::new(s) {
            Ok(regex) => Ok(Matcher(regex)),
            Err(_) => Err(serde::de::Error::custom(format!("Invalid regex: {}", s))),
        }
    }
}
