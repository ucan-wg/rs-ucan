// use super::enriched::Enriched;
use crate::{
    ability::arguments,
    invocation::promise::{Pending, Promise, PromiseAny, PromiseErr, PromiseOk, Resolves},
    url,
};
use enum_as_inner::EnumAsInner;
use libipld_core::{cid::Cid, ipld::Ipld};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

/// A recursive data structure whose leaves may be [`Ipld`] or promises.
///
/// [`Promised`] resolves to regular [`Ipld`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EnumAsInner)]
pub enum Promised {
    // Resolved Leaves
    Null,
    Bool(bool),
    Integer(i128),
    Float(f64),
    String(String),
    Bytes(Vec<u8>),
    Link(Cid),

    // Pending Leaves
    WaitOk(Cid),
    WaitErr(Cid),
    WaitAny(Cid),

    // Recursive
    List(Vec<Promised>),
    Map(BTreeMap<String, Promised>),
}

impl Promised {
    pub fn with_resolved<F, T>(self, f: F) -> Result<T, Pending>
    where
        F: FnOnce(Ipld) -> T,
    {
        match self.try_into() {
            Ok(ipld) => Ok(f(ipld)),
            Err(pending) => Err(pending),
        }
    }

    pub fn with_pending<F, E>(self, f: F) -> Result<E, Ipld>
    where
        F: FnOnce(Pending) -> E,
    {
        match self.try_into() {
            Ok(ipld) => Err(ipld),
            Err(promised) => Ok(f(promised)),
        }
    }
}

impl From<Ipld> for Promised {
    fn from(ipld: Ipld) -> Promised {
        match ipld {
            Ipld::Null => Promised::Null,
            Ipld::Bool(b) => Promised::Bool(b),
            Ipld::Integer(i) => Promised::Integer(i),
            Ipld::Float(f) => Promised::Float(f),
            Ipld::String(s) => Promised::String(s),
            Ipld::Bytes(b) => Promised::Bytes(b),
            Ipld::Link(cid) => Promised::Link(cid),
            Ipld::List(list) => Promised::List(list.into_iter().map(Into::into).collect()),
            Ipld::Map(map) => {
                let mut promised_map = BTreeMap::new();
                for (k, v) in map {
                    promised_map.insert(k, v.into());
                }
                Promised::Map(promised_map)
            }
        }
    }
}

impl TryFrom<Promised> for Ipld {
    type Error = Pending;

    fn try_from(promised: Promised) -> Result<Ipld, Self::Error> {
        match promised {
            Promised::Null => Ok(Ipld::Null),
            Promised::Bool(b) => Ok(Ipld::Bool(b)),
            Promised::Integer(i) => Ok(Ipld::Integer(i)),
            Promised::Float(f) => Ok(Ipld::Float(f)),
            Promised::String(s) => Ok(Ipld::String(s)),
            Promised::Bytes(b) => Ok(Ipld::Bytes(b)),
            Promised::Link(cid) => Ok(Ipld::Link(cid)),
            Promised::List(list) => list
                .into_iter()
                .try_fold(Vec::new(), |mut acc, promised| {
                    acc.push(promised.try_into()?);
                    Ok(acc)
                })
                .map(Ipld::List),
            Promised::Map(map) => map
                .into_iter()
                .try_fold(BTreeMap::new(), |mut acc, (k, v)| {
                    acc.insert(k, v.try_into()?);
                    Ok(acc)
                })
                .map(Ipld::Map),
            Promised::WaitOk(cid) => Err(Pending::Ok(cid).into()),
            Promised::WaitErr(cid) => Err(Pending::Err(cid).into()),
            Promised::WaitAny(cid) => Err(Pending::Any(cid).into()),
        }
    }
}

impl From<PromiseOk<Ipld>> for Promised {
    fn from(p_ok: PromiseOk<Ipld>) -> Promised {
        match p_ok {
            PromiseOk::Fulfilled(ipld) => ipld.into(),
            PromiseOk::Pending(cid) => Promised::WaitOk(cid),
        }
    }
}

impl From<PromiseErr<Ipld>> for Promised {
    fn from(p_err: PromiseErr<Ipld>) -> Promised {
        match p_err {
            PromiseErr::Rejected(ipld) => ipld.into(),
            PromiseErr::Pending(cid) => Promised::WaitErr(cid),
        }
    }
}

impl From<PromiseAny<Ipld, Ipld>> for Promised {
    fn from(p_any: PromiseAny<Ipld, Ipld>) -> Promised {
        match p_any {
            PromiseAny::Fulfilled(ipld) => ipld.into(),
            PromiseAny::Rejected(ipld) => ipld.into(),
            PromiseAny::Pending(cid) => Promised::WaitAny(cid),
        }
    }
}

impl From<Promise<Ipld, Ipld>> for Promised {
    fn from(promise: Promise<Ipld, Ipld>) -> Promised {
        match promise {
            Promise::Ok(p_ok) => p_ok.into(),
            Promise::Err(p_err) => p_err.into(),
            Promise::Any(p_any) => p_any.into(),
        }
    }
}

impl<T> From<Resolves<T>> for Promised
where
    Promised: From<T>,
{
    fn from(r: Resolves<T>) -> Promised {
        match r {
            Resolves::Ok(p_ok) => match p_ok {
                PromiseOk::Fulfilled(val) => val.into(),
                PromiseOk::Pending(cid) => Promised::WaitOk(cid),
            },
            Resolves::Err(p_err) => match p_err {
                PromiseErr::Rejected(val) => val.into(),
                PromiseErr::Pending(cid) => Promised::WaitErr(cid),
            },
        }
    }
}

impl<T> From<arguments::Named<T>> for Promised
where
    Promised: From<T>,
{
    fn from(args: arguments::Named<T>) -> Promised {
        Promised::Map(
            args.into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect::<BTreeMap<String, Promised>>(),
        )
    }
}

impl From<PathBuf> for Promised {
    fn from(path: PathBuf) -> Promised {
        Promised::String(path.to_string_lossy().to_string())
    }
}

impl From<Cid> for Promised {
    fn from(cid: Cid) -> Promised {
        Promised::Link(cid)
    }
}

impl From<::url::Url> for Promised {
    fn from(url: ::url::Url) -> Promised {
        Promised::String(url.to_string())
    }
}

impl From<url::Newtype> for Promised {
    fn from(nt: url::Newtype) -> Promised {
        nt.0.into()
    }
}

impl<T> From<Option<T>> for Promised
where
    Promised: From<T>,
{
    fn from(opt: Option<T>) -> Promised {
        match opt {
            Some(val) => val.into(),
            None => Promised::Null,
        }
    }
}

impl From<String> for Promised {
    fn from(s: String) -> Promised {
        Promised::String(s)
    }
}

impl From<f64> for Promised {
    fn from(f: f64) -> Promised {
        Promised::Float(f)
    }
}

impl From<i128> for Promised {
    fn from(i: i128) -> Promised {
        Promised::Integer(i)
    }
}

impl From<bool> for Promised {
    fn from(b: bool) -> Promised {
        Promised::Bool(b)
    }
}

impl From<Vec<u8>> for Promised {
    fn from(b: Vec<u8>) -> Promised {
        Promised::Bytes(b)
    }
}

impl<T> From<BTreeMap<String, T>> for Promised
where
    Promised: From<T>,
{
    fn from(map: BTreeMap<String, T>) -> Promised {
        Promised::Map(
            map.into_iter()
                .map(|(k, v)| (k, v.into()))
                .collect::<BTreeMap<String, Promised>>(),
        )
    }
}
impl<T> From<Vec<T>> for Promised
where
    Promised: From<T>,
{
    fn from(list: Vec<T>) -> Promised {
        Promised::List(list.into_iter().map(Into::into).collect())
    }
}

/***************************
| POST ORDER IPLD ITERATOR |
***************************/

/// A post-order [`Ipld`] iterator
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde-codec", derive(serde::Serialize))]
#[allow(clippy::module_name_repetitions)]
pub struct PostOrderIpldIter<'a> {
    inbound: Vec<&'a Promised>,
    outbound: Vec<&'a Promised>,
}

// #[derive(Clone, Debug, PartialEq)]
// pub enum Item<'a> {
//     Node(&'a Promised),
//     Inner(&'a Cid),
// }

impl<'a> PostOrderIpldIter<'a> {
    /// Initialize a new [`PostOrderIpldIter`]
    #[must_use]
    pub fn new(promised: &'a Promised) -> Self {
        PostOrderIpldIter {
            inbound: vec![promised],
            outbound: vec![],
        }
    }
}

impl<'a> Iterator for PostOrderIpldIter<'a> {
    type Item = &'a Promised;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.inbound.pop() {
                None => return self.outbound.pop(),
                Some(ref map @ Promised::Map(ref btree)) => {
                    self.outbound.push(map.clone());

                    for node in btree.values() {
                        self.inbound.push(node);
                    }
                }

                Some(ref list @ Promised::List(ref vector)) => {
                    self.outbound.push(list.clone());

                    for node in vector {
                        self.inbound.push(node);
                    }
                }
                Some(node) => self.outbound.push(node),
            }
        }
    }
}
