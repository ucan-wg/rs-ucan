use super::traits::Condition;
use libipld_core::{ipld::Ipld, serde as ipld_serde};
use regex::Regex;
use serde;
use serde_derive::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)]
pub enum Common {
    ContainsAll(ContainsAll),
    ContainsAny(ContainsAny),
    ExcludesAll(ExcludesAll),
    MaxLength(MaxLength),
    MinNumber(MinNumber),
    MaxNumber(MaxNumber),
    Matches(Matches),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContainsAll {
    field: String,
    values: Vec<Ipld>,
}

impl From<ContainsAll> for Ipld {
    fn from(contains_all: ContainsAll) -> Self {
        contains_all.into()
    }
}

impl TryFrom<Ipld> for ContainsAll {
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, ()> {
        ipld_serde::from_ipld(ipld).map_err(|_| ())
    }
}

impl Condition for ContainsAll {
    fn validate(&self, ipld: &Ipld) -> bool {
        match ipld {
            Ipld::List(array) => self.values.iter().all(|ipld| array.contains(ipld)),
            Ipld::Map(btree) => {
                let vals: Vec<&Ipld> = btree.values().collect();
                self.values.iter().all(|ipld| vals.contains(&ipld))
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContainsAny {
    field: String,
    value: Vec<Ipld>,
}

impl Condition for ContainsAny {
    fn validate(&self, ipld: &Ipld) -> bool {
        match ipld {
            Ipld::List(array) => array.iter().any(|ipld| self.value.contains(ipld)),
            Ipld::Map(btree) => {
                let vals: Vec<&Ipld> = btree.values().collect();
                self.value.iter().any(|ipld| vals.contains(&ipld))
            }
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExcludesAll {
    field: String,
    value: Vec<Ipld>,
}

impl Condition for ExcludesAll {
    fn validate(&self, ipld: &Ipld) -> bool {
        match ipld {
            Ipld::List(array) => self.value.iter().all(|ipld| !array.contains(ipld)),
            Ipld::Map(btree) => {
                let vals: Vec<&Ipld> = btree.values().collect();
                self.value.iter().all(|ipld| !vals.contains(&ipld))
            }
            _ => false,
        }
    }
}

impl TryFrom<Ipld> for ExcludesAll {
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld).map_err(|_| ())
    }
}

// FIXME serialization?
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub enum Numeric {
    Float(f64),
    Integer(i128),
}

impl TryFrom<Ipld> for Numeric {
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld).map_err(|_| ())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MinNumber {
    field: String,
    value: Numeric,
}

impl TryFrom<Ipld> for MinNumber {
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld).map_err(|_| ())
    }
}

impl Condition for MinNumber {
    fn validate(&self, ipld: &Ipld) -> bool {
        match ipld {
            Ipld::Integer(integer) => match self.value {
                Numeric::Float(float) => *integer as f64 >= float,
                Numeric::Integer(integer) => integer >= integer,
            },
            Ipld::Float(float) => match self.value {
                Numeric::Float(float) => float >= float,
                Numeric::Integer(integer) => *float >= integer as f64, // FIXME this needs tests
            },
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaxNumber {
    field: String,
    value: Numeric,
}

impl Condition for MaxNumber {
    fn validate(&self, ipld: &Ipld) -> bool {
        match ipld {
            Ipld::Integer(integer) => match self.value {
                Numeric::Float(float) => *integer as f64 <= float,
                Numeric::Integer(integer) => integer <= integer,
            },
            Ipld::Float(float) => match self.value {
                Numeric::Float(float) => float <= float,
                Numeric::Integer(integer) => *float <= integer as f64, // FIXME this needs tests
            },
            _ => false,
        }
    }
}

impl TryFrom<Ipld> for MaxNumber {
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld).map_err(|_| ())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MinLength {
    field: String,
    value: u64,
}

impl Condition for MinLength {
    fn validate(&self, ipld: &Ipld) -> bool {
        match ipld {
            Ipld::String(string) => string.len() >= self.value as usize,
            Ipld::List(list) => list.len() >= self.value as usize,
            Ipld::Map(btree) => btree.len() >= self.value as usize,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MaxLength {
    field: String,
    value: u64,
}

impl Condition for MaxLength {
    fn validate(&self, ipld: &Ipld) -> bool {
        match ipld {
            Ipld::String(string) => string.len() <= self.value as usize,
            Ipld::List(list) => list.len() <= self.value as usize,
            Ipld::Map(btree) => btree.len() <= self.value as usize,
            _ => false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Matches {
    field: String,
    matcher: Matcher,
}

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
            Err(_) => {
                // FIXME
                todo!()
            }
        }
    }
}

impl Condition for Matcher {
    fn validate(&self, ipld: &Ipld) -> bool {
        match ipld {
            Ipld::String(string) => self.0.is_match(string),
            _ => false,
        }
    }
}
