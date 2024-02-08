//! A [`Condition`] for ensuring a field contains none of a set of values.
use super::traits::Condition;
use crate::ability::arguments;
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde_derive::{Deserialize, Serialize};

/// A [`Condition`] for ensuring a field contains none of a set of values.
///
/// This works on all [`Ipld`] types. For lists and maps, it checks the values, not keys.
/// For the rest, it checks the value itself.
///
/// # Examples
///
/// ```rust
/// # use ucan::delegation::{excludes_all::ExcludesAll, traits::Condition};
/// # use libipld::ipld;
/// #
/// let args = ipld!({"a": [1, "b", 3.14], "b": 4}).try_into().unwrap();
/// let cond = ExcludesAll {
///     field: "a".into(),
///     excludes_all: vec![ipld!(2), ipld!("a")]
/// };
///
/// assert!(cond.validate(&args));
///
/// // Fails when the values of a map match
/// assert!(!cond.validate(&ipld!({"a": {"b": 2}}).try_into().unwrap()));
///
/// // Succeeds when the key is not present
/// assert!(ExcludesAll {
///     field: "nope".into(),
///     excludes_all: vec![ipld!(1), ipld!("b")]
/// }.validate(&args));
///
/// // Also checks non-maps/non-lists
/// assert!(!cond.validate(&ipld!({"a": 2}).try_into().unwrap()));
/// assert!(cond.validate(&ipld!({"a": "hello world"}).try_into().unwrap()));
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExcludesAll {
    /// Name of the field to check.
    pub field: String,

    /// The elements that must not be present.
    pub excludes_all: Vec<Ipld>,
}

impl From<ExcludesAll> for Ipld {
    fn from(excludes_all: ExcludesAll) -> Self {
        excludes_all.into()
    }
}

impl TryFrom<Ipld> for ExcludesAll {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl Condition for ExcludesAll {
    fn validate(&self, args: &arguments::Named) -> bool {
        if let Some(ipld) = args.get(&self.field) {
            let mut it = self.excludes_all.iter();
            match ipld {
                Ipld::Null => it.all(|x| x != ipld),
                Ipld::Bool(_) => it.all(|x| x != ipld),
                Ipld::Float(_) => it.all(|x| x != ipld),
                Ipld::Integer(_) => it.all(|x| x != ipld),
                Ipld::Bytes(_) => it.all(|x| x != ipld),
                Ipld::String(_) => it.all(|x| x != ipld),
                Ipld::Link(_) => it.all(|x| x != ipld),
                Ipld::List(array) => it.all(|x| !array.contains(x)),
                Ipld::Map(btree) => {
                    let vals: Vec<&Ipld> = btree.values().collect();
                    it.all(|x| !vals.contains(&x))
                }
            }
        } else {
            true
        }
    }
}
