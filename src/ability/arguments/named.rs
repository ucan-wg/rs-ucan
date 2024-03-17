use crate::{
    // ability::crud::update::TryFromIpldError,
    invocation::promise::{self, Pending},
    ipld,
};
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use js_sys::{Array, Map, Object, Reflect};

#[cfg(feature = "test_utils")]
use proptest::prelude::*;

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

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn values(&self) -> impl Iterator<Item = &T> {
        self.0.values()
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
        let btree: BTreeMap<String, T> = iter.into_iter().collect();
        Named(btree)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Error)]
pub enum FromIpldError<E> {
    NotAMap,
    LeafError(#[from] E),
}

impl<T: TryFrom<Ipld>> TryFrom<Ipld> for Named<T> {
    type Error = FromIpldError<<T as TryFrom<Ipld>>::Error>;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::Map(map) => Ok(map
                .into_iter()
                .try_fold(Named::new(), |mut named, (k, v)| {
                    let value = T::try_from(v).map_err(FromIpldError::LeafError)?;
                    named.insert(k, value);
                    Ok::<Named<T>, Self::Error>(named)
                })?),

            _ => Err(FromIpldError::NotAMap),
        }
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
    type Error = TryFromJsValueError;

    fn try_from(js: JsValue) -> Result<Self, Self::Error> {
        match T::try_from(js) {
            Err(()) => Err(TryFromJsValueError::NotIpld),
            Ok(Ipld::Map(map)) => Ok(Named(map)),
            Ok(_wrong_ipld) => Err(TryFromJsValueError::NotAMap),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Error)]
pub enum TryFromJsValueError {
    #[error("Not a map")]
    NotAMap,

    #[error("Not Ipld")]
    NotIpld,
}

impl From<Named<Ipld>> for Named<promise::Any<ipld::Promised>> {
    fn from(named: Named<Ipld>) -> Named<promise::Any<ipld::Promised>> {
        let btree: BTreeMap<String, promise::Any<ipld::Promised>> = named
            .into_iter()
            .map(|(k, v)| (k, promise::Any::from_ipld(v)))
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
    type Error = Pending;

    fn try_from(named: Named<ipld::Promised>) -> Result<Self, Self::Error> {
        named.iter().try_fold(Named::new(), |mut acc, (ref k, v)| {
            let ipld = v.clone().try_into()?;
            acc.insert(k.to_string(), ipld);
            Ok(acc)
        })
    }
}

impl<T: Clone> TryFrom<promise::Any<Named<T>>> for Named<Ipld>
where
    Ipld: TryFrom<T>,
{
    type Error = promise::Any<Named<T>>;

    fn try_from(resolves: promise::Any<Named<T>>) -> Result<Self, Self::Error> {
        resolves
            .clone()
            .try_resolve()?
            .into_iter()
            .try_fold(Named::new(), |mut btree, (k, v)| {
                let ipld = v.try_into().map_err(|_| ())?;
                btree.insert(k, ipld);
                Ok(btree)
            })
            .map_err(|_: ()| resolves) // FIXME
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

#[cfg(feature = "test_utils")]
impl Arbitrary for Named<Ipld> {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        prop::collection::btree_map(".*", ipld::Newtype::arbitrary(), 0..20)
            .prop_map(|newtype_map| {
                newtype_map
                    .into_iter()
                    .fold(Named::new(), |mut named, (k, v)| {
                        named.insert(k, v.0);
                        named
                    })
            })
            .boxed()
    }
}
