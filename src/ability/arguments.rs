//! Utilities for ability arguments

use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

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
/// # use ucan::ability::arguments;
/// # use url::Url;
/// # use libipld::{ipld, ipld::Ipld};
/// #
/// struct Execute {
///    program: Url,
///    instructions: arguments::Named<Ipld>,
/// }
///
/// let ability = Execute {
///   program: Url::parse("file://host.name/path/to/exe").unwrap(),
///   instructions: arguments::Named::from_iter([
///     ("bold".into(), ipld!(true)),
///     ("message".into(), ipld!("hello world")),
///   ])
/// };
///
/// assert_eq!(ability.instructions.get("bold"), Some(&ipld!(true)));
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Named<T>(pub BTreeMap<String, T>);

impl<T> Named<T> {
    /// Create a new, empty `Named` instance.
    pub fn new() -> Self {
        Default::default()
    }

    /// Get the value associated with a key.
    ///
    /// An alias for [`BTreeMap::insert`].
    pub fn get(&self, key: &str) -> Option<&T> {
        self.0.get(key)
    }

    /// Inserts a key-value pair.
    ///
    /// An alias for [`BTreeMap::insert`].
    pub fn insert(&mut self, key: String, value: T) -> Option<T> {
        self.0.insert(key, value)
    }

    /// Gets an iterator over the entries, sorted by key.
    ///
    /// A wrapper around [`BTreeMap::iter`].
    pub fn iter(&self) -> impl Iterator<Item = (&String, &T)> {
        self.0.iter()
    }

    /// The number of entries in.
    ///
    /// A wrapper around [`BTreeMap::len`].
    pub fn len(&self) -> usize {
        self.0.len()
    }
}

impl<T> Default for Named<T> {
    fn default() -> Self {
        Named(BTreeMap::new())
    }
}

impl<T> IntoIterator for Named<T> {
    type Item = (String, T);
    type IntoIter = std::collections::btree_map::IntoIter<String, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<T> FromIterator<(String, T)> for Named<T> {
    fn from_iter<I: IntoIterator<Item = (String, T)>>(iter: I) -> Self {
        Named(iter.into_iter().collect())
    }
}

impl<T: for<'de> Deserialize<'de>> TryFrom<Ipld> for Named<T> {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl<T: Serialize> From<Named<T>> for Ipld {
    fn from(arguments: Named<T>) -> Self {
        ipld_serde::to_ipld(arguments).unwrap()
    }
}

#[cfg(target_arch = "wasm32")]
impl<T: Into<JsValue>> From<Named<T>> for Object {
    fn from(arguments: Named<T>) -> Self {
        let obj = Object::new();
        for (k, v) in arguments.0 {
            Reflect::set(&obj, &k.into(), v.into()).unwrap();
        }
        obj
    }
}

#[cfg(target_arch = "wasm32")]
impl From<Named<Ipld>> for Object {
    fn from(arguments: Named<Ipld>) -> Self {
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
impl<T: TryFrom<JsValue>> From<&Object> for Named<T> {
    // FIXME probbaly needs to be a try_from
    fn from(obj: &Object) -> Self {
        let btree = Object::entries(obj)
            .iter()
            .map(|entry| {
                let entry = Array::from(&entry);
                let key = entry.get(0).as_string().unwrap(); // FIXME
                let value = T::try_from(entry.get(1)).unwrap().0; // FIXME
                (key, value)
            })
            .collect::<BTreeMap<String, Ipld>>();

        Named(btree)
    }
}

// NOTE saves a few cycles while calling by not cloning
// the extra Object fields that we're not going to use
#[cfg(target_arch = "wasm32")]
impl From<&Object> for Named<Ipld> {
    // FIXME probbaly needs to be a try_from
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
impl<T> From<Named<T>> for JsValue {
    fn from(arguments: Named<T>) -> Self {
        arguments
            .0
            .iter()
            .fold(Map::new(), |map, (ref k, v)| {
                map.set(&JsValue::from_str(k), &JsValue::from(v.clone()));
                map
            })
            .into()
    }
}

#[cfg(target_arch = "wasm32")]
impl From<Named<Ipld>> for JsValue {
    fn from(arguments: Named<Ipld>) -> Self {
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
impl<T> TryFrom<JsValue> for Named<T> {
    type Error = (); // FIXME

    fn try_from(js: JsValue) -> Result<Self, Self::Error> {
        match T::try_from(js) {
            Err(()) => Err(()), // FIXME surface that we can't parse at all
            Ok(Ipld::Map(map)) => Ok(Named(map)),
            Ok(_wrong_ipld) => Err(()), // FIXME surface that we have the wrong type
        }
    }
}

#[cfg(target_arch = "wasm32")]
impl TryFrom<JsValue> for Named<Ipld> {
    type Error = (); // FIXME

    fn try_from(js: JsValue) -> Result<Self, Self::Error> {
        match ipld::Newtype::try_from(js).map(|newtype| newtype.0) {
            Err(()) => Err(()), // FIXME surface that we can't parse at all
            Ok(Ipld::Map(map)) => Ok(Named(map)),
            Ok(_wrong_ipld) => Err(()), // FIXME surface that we have the wrong type
        }
    }
}

/// Errors for [`arguments::Named`][Named].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Error)]
pub enum NamedError {
    /// A required field was missing.
    #[error("Missing arguments::Named field {0}")]
    FieldMissing(String),

    /// The value at the named field didn't match the expected value.
    #[error("arguments::Named field {0}: value doesn't match")]
    FieldValueMismatch(String),
}
