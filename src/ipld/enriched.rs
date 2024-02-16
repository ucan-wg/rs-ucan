use crate::invocation::Resolvable;
use libipld_core::{cid::Cid, ipld::Ipld};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Enriched<T> {
    /// Lifted [`Ipld::Null`]
    Null,

    /// Lifted [`Ipld::Bool`]
    Bool(bool),

    /// Lifted [`Ipld::Integer`]
    Integer(i128),

    /// Lifted [`Ipld::Float`]
    Float(f64),

    /// Lifted [`Ipld::String`]
    String(String),

    /// Lifted [`Ipld::Bytes`] (byte array)
    Bytes(Vec<u8>),

    /// [`Ipld::List`], but where the values are [`PromiseIpld`].
    List(Vec<T>),

    /// [`Ipld::Map`], but where the values are [`PromiseIpld`].
    Map(BTreeMap<String, T>),

    /// Lifted [`Ipld::Link`]
    Link(Cid),
}

/// A post-order [`Ipld`] iterator
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde-codec", derive(serde::Serialize))]
#[allow(clippy::module_name_repetitions)]
pub struct PostOrderIpldIter<'a, T> {
    inbound: Vec<Item<'a, T>>,
    outbound: Vec<Item<'a, T>>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Item<'a, T> {
    Node(&'a Enriched<T>),
    Inner(&'a T),
}

impl<'a, T> PostOrderIpldIter<'a, T> {
    /// Initialize a new [`PostOrderIpldIter`]
    #[must_use]
    pub fn new(enriched: &'a Enriched<T>) -> Self {
        PostOrderIpldIter {
            inbound: vec![Item::Node(enriched)],
            outbound: vec![],
        }
    }
}

impl<'a, T> IntoIterator for &'a Enriched<T> {
    type Item = Item<'a, T>;
    type IntoIter = PostOrderIpldIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        PostOrderIpldIter::new(&self)
    }
}

impl<'a, T: Clone> FromIterator<Item<'a, T>> for &'a Enriched<T> {
    fn from_iter<I: IntoIterator<Item = Item<'a, T>>>(it: I) -> Self {
        &it.into_iter().fold(Enriched::Null, |acc, x| match x {
            Item::Node(Enriched::Null) => Enriched::Null,
            Item::Node(Enriched::Bool(b)) => Enriched::Bool(*b),
            Item::Node(Enriched::Integer(i)) => Enriched::Integer(*i),
            Item::Node(Enriched::Float(f)) => Enriched::Float(*f),
            Item::Node(Enriched::String(s)) => Enriched::String(s.clone()),
            Item::Node(Enriched::Bytes(b)) => Enriched::Bytes(b.clone()),
            Item::Node(Enriched::Link(c)) => Enriched::Link(c.clone()),
            Item::Node(Enriched::List(vec)) => {
                let mut list = vec![];
                for item in vec {
                    list.push(item);
                }
                Enriched::List(list.iter().map(|a| (*a).clone()).collect())
            }
            Item::Node(Enriched::Map(btree)) => {
                let mut map = BTreeMap::new();
                for (k, v) in btree {
                    map.insert(k.clone(), (*v).clone());
                }
                Enriched::Map(map)
            }
            Item::Inner(_) => acc,
        })
    }
}

impl<'a, T> From<&'a Enriched<T>> for PostOrderIpldIter<'a, T> {
    fn from(enriched: &'a Enriched<T>) -> Self {
        PostOrderIpldIter::new(enriched)
    }
}

impl<'a, T> Iterator for PostOrderIpldIter<'a, T> {
    type Item = Item<'a, T>;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inbound.pop() {
                None => return self.outbound.pop(),
                Some(map @ Item::Node(Enriched::Map(btree))) => {
                    self.outbound.push(map);

                    for node in btree.values() {
                        self.inbound.push(Item::Inner(node));
                    }
                }

                Some(list @ Item::Node(Enriched::List(vector))) => {
                    self.outbound.push(list);

                    for node in vector {
                        self.inbound.push(Item::Inner(node));
                    }
                }
                Some(node) => self.outbound.push(node),
            }
        }
    }
}

impl<T: From<Ipld>> From<Ipld> for Enriched<T> {
    fn from(ipld: Ipld) -> Self {
        match ipld {
            Ipld::Null => Enriched::Null,
            Ipld::Bool(b) => Enriched::Bool(b),
            Ipld::Integer(i) => Enriched::Integer(i),
            Ipld::Float(f) => Enriched::Float(f),
            Ipld::String(s) => Enriched::String(s),
            Ipld::Bytes(b) => Enriched::Bytes(b),
            Ipld::List(l) => Enriched::List(l.into_iter().map(From::from).collect()),
            Ipld::Map(m) => Enriched::Map(m.into_iter().map(|(k, v)| (k, From::from(v))).collect()),
            Ipld::Link(c) => Enriched::Link(c),
        }
    }
}

impl<T: Clone + TryInto<Ipld>> TryFrom<Enriched<T>> for Ipld {
    type Error = Enriched<T>;

    fn try_from(enriched: Enriched<T>) -> Result<Ipld, Self::Error> {
        match enriched {
            Enriched::List(ref vec) => {
                let result: Result<Vec<Ipld>, ()> = vec.iter().try_fold(vec![], |mut acc, x| {
                    let resolved = x.clone().try_into().map_err(|_| ())?;
                    acc.push(resolved);
                    Ok(acc)
                });

                match result {
                    Ok(vec) => Ok(vec.into()),
                    Err(()) => Err(enriched),
                }
            }
            Enriched::Map(ref btree) => {
                let result: Result<BTreeMap<String, Ipld>, ()> =
                    btree.iter().try_fold(BTreeMap::new(), |mut acc, (k, v)| {
                        let resolved = v.clone().try_into().map_err(|_| ())?;
                        acc.insert(k.clone(), resolved);
                        Ok(acc)
                    });

                match result {
                    Ok(vec) => Ok(vec.into()),
                    Err(()) => Err(enriched),
                }
            }
            Enriched::Null => Ok(Ipld::Null),
            Enriched::Bool(b) => Ok(b.into()),
            Enriched::Integer(i) => Ok(i.into()),
            Enriched::Float(f) => Ok(f.into()),
            Enriched::String(s) => Ok(s.into()),
            Enriched::Bytes(b) => Ok(b.into()),
            Enriched::Link(l) => Ok(l.into()),
        }
    }
}
