use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize, Serializer};
use std::collections::BTreeMap;

// FIXME rename modeule to metadata

pub trait Entry {
    const KEY: &'static str;
}

pub trait Entries: TryFrom<Ipld> + Into<Ipld> {
    const KEYS: &'static [&'static str];
}

pub trait Meta {}
impl<T: Entries> Meta for T {}

pub enum Empty {}
impl Meta for Empty {}

// NOTE no Serde
#[derive(Debug, Clone, PartialEq)]
pub struct Metadata<T: Meta> {
    known: BTreeMap<String, T>,
    unknown: BTreeMap<String, Ipld>,
}

impl From<Metadata<Empty>> for Ipld {
    fn from(meta: Metadata<Empty>) -> Ipld {
        Ipld::Map(meta.unknown)
    }
}

impl TryFrom<Ipld> for Metadata<Empty> {
    type Error = ();

    fn try_from(ipld: Ipld) -> Result<Metadata<Empty>, Self::Error> {
        match ipld {
            Ipld::Map(unknown) => Ok(Metadata {
                known: BTreeMap::new(),
                unknown,
            }),
            _ => Err(()),
        }
    }
}

impl<T: Entries> Serialize for Metadata<T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = Ipld::from(*self); // FIXME kill that clone with tons of refs?
        serde::Serialize::serialize(&s, serializer)
    }
}

impl<'de, T: Entries + Clone> Deserialize<'de> for Metadata<T> {
    fn deserialize<D>(d: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        Ipld::deserialize(d).map(Metadata::from)
    }
}

impl<E: Entries> TryFrom<Metadata<E>> for Ipld {
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

impl<T: Entries> TryFrom<Ipld> for Metadata<T> {
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

impl<T: Meta + Clone> Metadata<T> {
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

    pub fn known<'a>(&'a self) -> &'a BTreeMap<String, T> {
        &self.known
    }

    pub fn unknown<'a>(&'a self) -> &'a BTreeMap<String, Ipld> {
        &self.unknown
    }

    // FIXME types
    pub fn insert_known<'a>(&'a mut self, key: String, value: T) -> Result<(), Option<T>> {
        if let Some(_) = self.unknown.get(&key) {
            return Err(None);
        }

        match self.known.insert(key, value) {
            Some(v) => Err(Some(v)),
            _ => Ok(()),
        }
    }

    pub fn insert_unknown<'a>(&'a mut self, key: String, value: Ipld) -> Result<(), Option<Ipld>> {
        if let Some(_) = self.known.get(&key) {
            return Err(None);
        }

        match self.unknown.insert(key, value) {
            Some(v) => Err(Some(v)),
            _ => Ok(()),
        }
    }
}

pub trait Mergable {
    fn merge(&self) -> BTreeMap<String, Ipld>;
    fn extract(merged: BTreeMap<String, Ipld>) -> Self;
}

impl Mergable for Metadata<Empty> {
    fn merge(&self) -> BTreeMap<String, Ipld> {
        self.unknown
    }

    // FIXME better error
    fn extract(unknown: BTreeMap<String, Ipld>) -> Self {
        Metadata {
            known: BTreeMap::new(),
            unknown,
        }
    }
}

impl<T: Entries + Clone> Mergable for Metadata<T> {
    fn merge(&self) -> BTreeMap<String, Ipld> {
        let mut meta = self.unknown().clone();

        for (k, v) in self.known() {
            meta.insert(k.clone(), v.clone().into());
        }

        meta
    }

    // FIXME better error
    fn extract(merged: BTreeMap<String, Ipld>) -> Self {
        let mut known = BTreeMap::new();
        let mut unknown = BTreeMap::new();

        for (k, v) in merged {
            if let Ok(entry) = v.clone().try_into() {
                known.insert(k, entry);
            } else {
                unknown.insert(k, v);
            }
        }

        Metadata { known, unknown }
    }
}

impl<T: Meta> Default for Metadata<T> {
    fn default() -> Self {
        Metadata {
            known: BTreeMap::new(),
            unknown: BTreeMap::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct IpvmConfig {
    pub max_retries: u32,
    pub workflow_fuel: u32,
}

impl Entry for IpvmConfig {
    const KEY: &'static str = "ipvm/config";
}

impl From<IpvmConfig> for Ipld {
    fn from(config: IpvmConfig) -> Self {
        config.into()
    }
}

impl TryFrom<Ipld> for IpvmConfig {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}
