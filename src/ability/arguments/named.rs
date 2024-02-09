use crate::{invocation::promise, ipld};
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
/// [`Named`] is a newtype wrapper around unstructured named args: `BTreeMap<String, T>`.
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

    pub fn contains(&self, other: &Named<T>) -> Result<(), NamedError>
    where
        T: PartialEq,
    {
        // `other` should usually be smaller than `self`
        for (k, other_v) in other.iter() {
            if let Some(self_v) = self.get(k) {
                if *self_v != *other_v {
                    return Err(NamedError::FieldValueMismatch(k.clone()));
                }
            } else {
                return Err(NamedError::FieldMissing(k.clone()));
            }
        }

        Ok(())
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

impl<T: Into<Ipld>> From<Named<T>> for Ipld {
    fn from(arguments: Named<T>) -> Self {
        Ipld::Map(
            arguments
                .0
                .into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect::<BTreeMap<String, Ipld>>(),
        )
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

use crate::invocation::promise::Resolves;

// impl<T: TryFrom<Ipld>> TryFrom<Named<T>> for Named<Ipld> {
//     type Error = ();
//
//     fn try_from(named: Named<T>) -> Result<Self, Self::Error> {
//         let btree = named
//             .0
//             .into_iter()
//             .map(|(k, v)| {
//                 let ipld = v.try_into().map_err(|_| ())?;
//                 Ok((k, ipld))
//             })
//             .collect::<Result<_, _>>()?;
//
//         Ok(Named(btree))
//     }
// }
// the trait `From<Named<Ipld>>` is not implemented for `Named<promised::Promised>`

// impl From<Named<Ipld>> for Named<Resolves<Ipld>> {
//     fn from(named: Named<Ipld>) -> Named<Resolves<Ipld>> {
//         named
//             .into_iter()
//             .map(|(k, v)| (k, promise::PromiseOk::Fulfilled(v).into()))
//             .collect()
//     }
// }
// impl<T: TryFrom<Ipld>> TryFrom<Named<Ipld>> for Named<Resolves<T>> {
//     type Error = Named<Ipld>;
//
//     fn try_from(named: Named<Ipld>) -> Result<Named<Resolves<T>>, Self::Error> {
//         named
//             .into_iter()
//             .try_fold(Named::new(), |mut btree, (k, v)| {
//                 let ipld = v.try_into().map_err(|_| named.clone())?;
//                 btree.insert(k, promise::PromiseOk::Fulfilled(ipld).into());
//                 Ok(btree)
//             })
//     }
// }

// FIXME abstract over both of these?
impl From<Named<Ipld>> for Named<Resolves<ipld::Promised>> {
    fn from(named: Named<Ipld>) -> Named<Resolves<ipld::Promised>> {
        let btree: BTreeMap<String, Resolves<ipld::Promised>> = named
            .into_iter()
            .map(|(k, v)| {
                let promised: ipld::Promised = v.into();
                (k, Resolves::new(promised))
            })
            .collect();

        Named(btree)
    }
}

impl From<Named<Ipld>> for Named<ipld::Promised> {
    fn from(named: Named<Ipld>) -> Named<ipld::Promised> {
        let btree: BTreeMap<String, ipld::Promised> =
            named.into_iter().map(|(k, v)| (k, v.into())).collect();

        Named(btree)
    }
}

impl TryFrom<Named<ipld::Promised>> for Named<Ipld> {
    type Error = Named<ipld::Promised>;

    fn try_from(named: Named<ipld::Promised>) -> Result<Self, Self::Error> {
        // FIXME lots of clone
        // FIXME idea: what if they implemet a is_resoled, and then the try_from?
        // This lets us check by ref, and then do the conversion and unwrap
        named
            .iter()
            .try_fold(Named::new(), |mut acc, (ref k, v)| {
                let ipld = v.clone().try_into().map_err(|_| ())?;
                acc.insert(k.to_string(), ipld);
                Ok(acc)
            })
            .map_err(|()| named.clone())
    }
}

impl<T: Clone> TryFrom<Resolves<Named<T>>> for Named<Ipld>
where
    Ipld: TryFrom<T>,
{
    type Error = Resolves<Named<T>>;

    fn try_from(resolves: Resolves<Named<T>>) -> Result<Self, Self::Error> {
        resolves
            .clone() // FIXME could be a pretty heavy clone
            .try_resolve()?
            .into_iter()
            .try_fold(Named::new(), |mut btree, (k, v)| {
                let ipld = v.try_into().map_err(|_| ())?;
                btree.insert(k, ipld);
                Ok(btree)
            })
            .map_err(|_: ()| resolves)
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
