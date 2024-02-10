//! A [`Condition`] for ensuring a map contains a key.
use super::traits::Condition;
use crate::ability::arguments;
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde_derive::{Deserialize, Serialize};

/// A [`Condition`] for ensuring a map contains a key.
///
/// Note that this operates on a key inside the args, not the args themselves.
/// The shape of an [`ability`][crate::ability] is pretermined, so further
/// constraining the top-level argument keys is not necessary.
///
/// # Examples
///
/// ```rust
/// # use ucan::delegation::{condition::{ContainsKey, Condition}};
/// # use libipld::ipld;
/// #
/// let args = ipld!({"a": {"b": 1, "c": 2}, "d": {"e": 3}}).try_into().unwrap();
/// let cond = ContainsKey{
///   field: "a".into(),
///   key: "b".into()
/// };
///
/// assert!(cond.validate(&args));
///
/// // Fails when the key is not present
/// assert!(!ContainsKey {
///     field: "nope".into(),
///     key: "b".into()
/// }.validate(&args));
///
/// // Also fails when the input is not a map
/// let list = ipld!({"a": [1, 2, 3]}).try_into().unwrap();
/// assert!(!cond.validate(&list));
/// assert!(!cond.validate(&ipld!({"a": 42}).try_into().unwrap()));
/// ```
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContainsKey {
    /// Name of the field to check.
    pub field: String,

    /// The elements that must be present.
    pub key: String,
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
    fn validate(&self, args: &arguments::Named<Ipld>) -> bool {
        match args.get(&self.field) {
            Some(Ipld::Map(map)) => map.contains_key(&self.key),
            _ => false,
        }
    }
}
