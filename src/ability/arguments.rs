//! Utilities for ability arguments

use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use js_sys::{Array, Map, Object, Reflect};

#[cfg(target_arch = "wasm32")]
use crate::ipld;

/// Named arguments
///
/// Being such a common pattern, but with so few trait implementations,
/// [`Named`] is a newtype wrapper around unstructured named args: `BTreeMap<String, Ipld>`.
///
/// # Examples
///
/// ```rust
/// # use ucan::ability::arguments::Named;
/// # use url::Url;
/// # use libipld::ipld;
/// #
/// struct Execute {
///    program: Url,
///    args: Named,
/// }
///
/// let ability = Execute {
///   program: Url::parse("file://host.name/path/to/exe").unwrap(),
///   args: Named::try_from(ipld!({
///     "bold": true,
///     "message": "hello world",
///   })).unwrap()
/// };
///
/// assert_eq!(ability.args.get("bold"), Some(&ipld!(true)));
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Named(pub BTreeMap<String, Ipld>);

impl Named {
    /// Get the value associated with a key
    ///
    /// An alias for [`BTreeMap::insert`].
    pub fn get(&self, key: &str) -> Option<&Ipld> {
        self.0.get(key)
    }

    /// Inserts a key-value pair
    ///
    /// An alias for [`BTreeMap::insert`].
    pub fn insert(&mut self, key: String, value: Ipld) -> Option<Ipld> {
        self.0.insert(key, value)
    }

    /// Gets an iterator over the entries, sorted by key.
    ///
    /// A wrapper around [`BTreeMap::iter`].
    pub fn iter(&self) -> impl Iterator<Item = (&String, &Ipld)> {
        self.0.iter()
    }
}

impl Default for Named {
    fn default() -> Self {
        Named(BTreeMap::new())
    }
}

impl IntoIterator for Named {
    type Item = (String, Ipld);
    type IntoIter = std::collections::btree_map::IntoIter<String, Ipld>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl FromIterator<(String, Ipld)> for Named {
    fn from_iter<T: IntoIterator<Item = (String, Ipld)>>(iter: T) -> Self {
        Named(iter.into_iter().collect())
    }
}

impl TryFrom<Ipld> for Named {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl From<Named> for Ipld {
    fn from(arguments: Named) -> Self {
        ipld_serde::to_ipld(arguments).unwrap()
    }
}

#[cfg(target_arch = "wasm32")]
impl From<Named> for Object {
    fn from(arguments: Named) -> Self {
        let obj = Object::new();
        for (k, v) in arguments.0 {
            Reflect::set(&obj, &k.into(), &ipld::Newtype(v).into()).unwrap();
        }
        obj
    }
}

// NOTE saves a few cycles while calling by not cloning
// the extra Object fields that we're not going to use
#[cfg(target_arch = "wasm32")]
impl From<&Object> for Named {
    fn from(obj: &Object) -> Self {
        let btree = Object::entries(obj)
            .iter()
            .map(|entry| {
                let entry = Array::from(&entry);
                let key = entry.get(0).as_string().unwrap(); // FIXME
                let value = ipld::Newtype::try_from(entry.get(1)).unwrap().0;
                (key, value)
            })
            .collect::<BTreeMap<String, Ipld>>();

        Named(btree)
    }
}

#[cfg(target_arch = "wasm32")]
impl From<Named> for JsValue {
    fn from(arguments: Named) -> Self {
        arguments
            .0
            .iter()
            .fold(Map::new(), |map, (ref k, v)| {
                map.set(
                    &JsValue::from_str(k),
                    &JsValue::from(ipld::Newtype(v.clone())),
                );
                map
            })
            .into()
    }
}

#[cfg(target_arch = "wasm32")]
impl TryFrom<JsValue> for Named {
    type Error = (); // FIXME

    fn try_from(js: JsValue) -> Result<Self, Self::Error> {
        match ipld::Newtype::try_from(js).map(|newtype| newtype.0) {
            Err(()) => Err(()), // FIXME surface that we can't parse at all
            Ok(Ipld::Map(map)) => Ok(Named(map)),
            Ok(_wrong_ipld) => Err(()), // FIXME surface that we have the wrong type
        }
    }
}
