//! A [`Condition`] for ensuring a field contains none of a set of keys.
use super::traits::Condition;
use crate::ability::arguments;
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde_derive::{Deserialize, Serialize};

/// A [`Condition`] for ensuring a map excludes a key.
///
/// Note that this operates on a key inside the args, not the args themselves.
/// The shape of an [`ability`][crate::ability] is pretermined, so further
/// constraining the top-level argument keys is not necessary.
///
/// # Examples
///
/// ```rust
/// # use ucan::delegation::{excludes_key::ExcludesKey, traits::Condition};
/// # use libipld::ipld;
/// #
/// let args = ipld!({"a": {"b": 1, "c": 2}, "d": {"e": 3}}).try_into().unwrap();
/// let cond = ExcludesKey{
///   field: "a".into(),
///   key: "b".into()
/// };
///
/// assert!(!cond.validate(&args));
///
/// // Succeeds when the key is not present
/// assert!(ExcludesKey {
///     field: "yep".into(),
///     key: "b".into()
/// }.validate(&args));
///
/// // Also succeeds when the input is not a map
/// let list = ipld!({"a": [1, 2, 3]}).try_into().unwrap();
/// assert!(cond.validate(&list));
/// assert!(cond.validate(&ipld!({"a": 42}).try_into().unwrap()));
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExcludesKey {
    /// Name of the field to check.
    pub field: String,

    /// The key that must not be present.
    pub key: String,
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
    fn validate(&self, args: &arguments::Named) -> bool {
        match args.get(&self.field) {
            Some(Ipld::Map(map)) => map.contains_key(&self.field),
            _ => true,
        }
    }
}
