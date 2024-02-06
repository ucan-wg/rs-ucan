//! Metadata (i.e. the UCAN `meta` field)

mod keyed;
pub use keyed::{Keyed, MultiKeyed};

use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize, Serializer};
use std::{collections::BTreeMap, convert::Infallible};

/// An uninhabited type that signals no known metadata fields.
///
/// This uses a similar technique as [`Infallible`];
/// it is not possible to create a runtime value of this type, so merely stubs out code paths.
///
/// ```
/// # use ucan::metadata::{Metadata, Empty};
/// # use std::collections::BTreeMap;
/// # use libipld_core::ipld::Ipld;
/// #
/// let kv: BTreeMap<String, Ipld> = BTreeMap::from_iter([
///    ("foo".into(), Ipld::String("hello world".to_string())),
///    ("bar".into(), Ipld::Integer(42)),
///    ("baz".into(), Ipld::List(vec![Ipld::Float(3.14)]))
/// ]);
///
/// let meta: Metadata<Empty> = Metadata::try_from(kv.clone()).unwrap();
///
/// assert_eq!(meta.known().len(), 0);
/// assert_eq!(meta.unknown(), &kv);
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Empty {}

/// A type alias for [`Metadata`] with no known fields.
pub type Unstructured = Metadata<Empty>;

// NOTE no Serde
/// Parsed metadata fields.
///
/// If you don't have any known fields, you can set `T ` to [`Empty`] (or [`Unstructured`])
#[derive(Debug, Clone, PartialEq)]
pub struct Metadata<T> {
    /// Structured metadata, selected by matching `T`
    known: BTreeMap<String, T>,

    /// Unstructured metadata
    unknown: BTreeMap<String, Ipld>,
}

impl<T> Metadata<T> {
    /// Constructor for [`Metadata`]
    ///
    /// This checks that no duplicate keys are present
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use ucan::metadata::{Metadata, Empty};
    /// # use std::collections::BTreeMap;
    /// # use libipld_core::ipld::Ipld;
    /// #
    /// #[derive(Debug, Clone, PartialEq)]
    /// pub enum MyFacts {
    ///    Timeout(u32),
    ///    Retry{max: u32, delay: u32},
    ///    NotGoingToUseThisOne(String),
    /// }
    ///
    /// let known: BTreeMap<String, MyFacts> = BTreeMap::from_iter([
    ///   ("timeout".into(), MyFacts::Timeout(1000)),
    ///   ("retry".into(), MyFacts::Retry{max: 5, delay: 100})
    /// ]);
    ///
    /// let unknown: BTreeMap<String, Ipld> = BTreeMap::from_iter([
    ///    ("foo".into(), Ipld::String("hello world".to_string())),
    ///    ("bar".into(), Ipld::Integer(42)),
    ///    ("baz".into(), Ipld::List(vec![Ipld::Float(3.14)]))
    /// ]);
    ///
    /// let meta = Metadata::new(known.clone(), unknown.clone()).unwrap();
    ///
    /// assert_eq!(meta.known(), &known.clone());
    /// assert_eq!(meta.unknown(), &unknown.clone());
    ///
    /// let collision: BTreeMap<String, Ipld> = BTreeMap::from_iter([
    ///    ("timeout".into(), Ipld::String("not a timeout".to_string())),
    /// ]);
    ///
    /// let meta = Metadata::new(known, collision);
    ///
    /// assert!(meta.is_err());
    /// ```
    pub fn new(
        known: BTreeMap<String, T>,
        unknown: BTreeMap<String, Ipld>,
    ) -> Result<Self, String> {
        for k in known.keys() {
            if unknown.contains_key(k) {
                return Err(k.into());
            }
        }

        Ok(Self { known, unknown })
    }

    /// Getter for the `known` field
    pub fn known<'a>(&'a self) -> &'a BTreeMap<String, T> {
        &self.known
    }

    /// Getter for the `unknown` field
    pub fn unknown<'a>(&'a self) -> &'a BTreeMap<String, Ipld> {
        &self.unknown
    }

    /// Insert a value into the `known` field
    ///
    /// This will return `Some(Entry::Unknown(ipld))` if you insert a key that already
    /// exists in the `unknown` field.
    ///
    /// It will return `Some(t: T)` if you insert a key that was already present
    /// in the `known` field.
    pub fn insert_known<'a>(&'a mut self, key: String, value: T) -> Option<Entry<T>> {
        if let Some(ipld) = self.unknown.get(&key) {
            self.known.insert(key, value);
            return Some(Entry::Unknown(ipld.clone()));
        }

        self.known.insert(key, value).map(Entry::Known)
    }

    /// Insert a value into the `unknown` field
    ///
    /// This will return `Some(Entry::Unknown(ipld))` if you insert a key that already
    /// exists in the `unknown` field.
    ///
    /// It will return `Some(t: T)` if you insert a key that was already present
    /// in the `known` field.
    pub fn insert_unknown<'a>(&'a mut self, key: String, value: Ipld) -> Option<Entry<T>>
    where
        T: Clone,
    {
        if let Some(t) = self.known.get(&key) {
            self.unknown.insert(key, value);
            return Some(Entry::Known(t.clone()));
        }

        self.unknown.insert(key, value).map(Entry::Unknown)
    }

    /// Remove a field from either field
    ///
    /// This will return `Some(Entry::Unknown(ipld))` if there was a key in the `unknown` field.
    ///
    /// It will return `Some(t: T)` if there was a key in the `known` field.
    pub fn remove_key<'a>(&'a mut self, key: &str) -> Option<Entry<T>> {
        if let Some(ipld) = self.unknown.remove(key) {
            return Some(Entry::Unknown(ipld));
        }

        self.known.remove(key).map(Entry::Known)
    }
}

/// Tag values as belonging to the `known` or `unknown` field.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Entry<T> {
    /// The tag for a value in the `known` field
    Known(T),

    /// The tag for a value in the `unknown` field
    Unknown(Ipld),
}

impl<T> Default for Metadata<T> {
    fn default() -> Self {
        Metadata {
            known: BTreeMap::new(),
            unknown: BTreeMap::new(),
        }
    }
}

impl From<Metadata<Empty>> for BTreeMap<String, Ipld> {
    fn from(meta: Metadata<Empty>) -> Self {
        meta.unknown
    }
}

impl<T: Into<Ipld>> From<Metadata<T>> for BTreeMap<String, Ipld> {
    // NOTE duplicate keys "shouldn't" be possible (because this roughly follows GDP)
    // ...so we can just merge
    fn from(meta: Metadata<T>) -> Self {
        let mut btree = meta.unknown;
        for (k, v) in meta.known {
            btree.insert(k, v.into());
        }
        btree
    }
}

impl<T: MultiKeyed> From<BTreeMap<String, Ipld>> for Metadata<T> {
    // FIXME better error
    fn from(merged: BTreeMap<String, Ipld>) -> Self {
        let mut known = BTreeMap::new();
        let mut unknown = BTreeMap::new();

        for (k, v) in merged {
            if T::KEYS.contains(&k.as_str()) {
                if let Ok(entry) = v.clone().try_into() {
                    known.insert(k, entry);
                } else {
                    unknown.insert(k, v);
                }
            } else {
                unknown.insert(k, v);
            }
        }

        Metadata { known, unknown }
    }
}

impl From<BTreeMap<String, Ipld>> for Metadata<Empty> {
    fn from(btree: BTreeMap<String, Ipld>) -> Self {
        Metadata {
            known: BTreeMap::new(),
            unknown: btree,
        }
    }
}

impl TryFrom<Metadata<Empty>> for Ipld {
    type Error = Infallible;

    fn try_from(meta: Metadata<Empty>) -> Result<Ipld, Infallible> {
        Ok(Ipld::Map(meta.unknown))
    }
}

impl<E: MultiKeyed + Into<Ipld>> TryFrom<Metadata<E>> for Ipld {
    type Error = String; // FIXME

    fn try_from(meta: Metadata<E>) -> Result<Self, Self::Error> {
        let mut btree = meta.unknown.clone();

        for (k, v) in meta.known {
            if let Some(_) = meta.unknown.get(&k) {
                return Err(k);
            }

            btree.insert(k, v.into());
        }

        Ok(Ipld::Map(btree))
    }
}

impl<T: MultiKeyed + Clone> Serialize for Metadata<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = Ipld::Map((*self).clone().into());
        serde::Serialize::serialize(&s, serializer)
    }
}

impl<'de, T: MultiKeyed + Clone> Deserialize<'de> for Metadata<T> {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ipld::deserialize(d).and_then(|ipld| ipld.try_into().map_err(|_| todo!()))
    }
}

impl<T: MultiKeyed> TryFrom<Ipld> for Metadata<T> {
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::Map(btree) => {
                let mut known = BTreeMap::new();
                let mut unknown = BTreeMap::new();

                for (k, v) in btree {
                    if T::KEYS.contains(&k.as_str()) {
                        if let Ok(fact) = T::try_from(v.clone()) {
                            known.insert(k, fact);
                        } else {
                            unknown.insert(k, v);
                        }
                    } else {
                        unknown.insert(k, v);
                    }
                }

                Ok(Self { known, unknown })
            }
            _ => Err(()),
        }
    }
}

// // FIXME Just as an example, plz delete
// #[derive(Debug, Clone, Serialize, Deserialize)]
// #[serde(deny_unknown_fields)]
// pub struct IpvmConfig {
//     pub max_retries: u32,
//     pub workflow_fuel: u32,
// }
//
// impl Keyed for IpvmConfig {
//     const KEY: &'static str = "ipvm/config";
// }
//
// impl From<IpvmConfig> for Ipld {
//     fn from(config: IpvmConfig) -> Self {
//         config.into()
//     }
// }
//
// impl TryFrom<Ipld> for IpvmConfig {
//     type Error = SerdeError;
//
//     fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
//         ipld_serde::from_ipld(ipld)
//     }
// }
