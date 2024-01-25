use super::Condition;
use libipld_core::ipld::Ipld;
use regex::Regex;
use std::collections::BTreeMap;

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

// FIXME dynamic js version?

#[derive(Debug, Clone, PartialEq)]
pub struct ContainsAll {
    field: String,
    values: Vec<Ipld>,
}

impl From<ContainsAll> for Ipld {
    fn from(contains_all: ContainsAll) -> Self {
        let mut map = BTreeMap::new();
        map.insert("field".into(), contains_all.field.into());
        map.insert("values".into(), contains_all.values.into());
        map.into()
    }
}

impl TryFrom<&Ipld> for ContainsAll {
    type Error = (); // FIXME

    fn try_from(ipld: &Ipld) -> Result<Self, ()> {
        if let Ipld::Map(map) = ipld {
            if map.len() != 2 {
                return Err(());
            }

            if let Some(Ipld::String(field)) = map.get("field") {
                let values = match map.get("values") {
                    None => Ok(vec![]),
                    Some(Ipld::List(values)) => Ok(values.to_vec()),
                    _ => Err(()),
                }?;

                Ok(Self {
                    field: field.to_string(),
                    values: values.to_vec(),
                })
            } else {
                Err(())
            }
        } else {
            Err(())
        }
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

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq)]
pub enum Numeric {
    Float(f64),
    Integer(i128),
}

#[derive(Debug, Clone, PartialEq)]
pub struct MinNumber {
    field: String,
    value: Numeric,
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

#[derive(Debug, Clone, PartialEq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq, Eq)]
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

#[derive(Debug, Clone, PartialEq)]
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

impl Condition for Matcher {
    fn validate(&self, ipld: &Ipld) -> bool {
        match ipld {
            Ipld::String(string) => self.0.is_match(string),
            _ => false,
        }
    }
}
