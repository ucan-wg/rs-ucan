use crate::{
    ability::arguments,
    invocation::promise::{self, Pending, PromiseErr, PromiseOk},
    url,
};
use enum_as_inner::EnumAsInner;
use libipld_core::{cid::Cid, ipld::Ipld};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt, path::PathBuf};

/// A recursive data structure whose leaves may be [`Ipld`] or promises.
///
/// [`Promised`] resolves to regular [`Ipld`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EnumAsInner)]
pub enum Promised {
    /// Resolved null.
    Null,

    /// Resolved Boolean.
    Bool(bool),

    /// Resolved integer.
    Integer(i128),

    /// Resolved float.
    Float(f64),

    /// Resolved string.
    String(String),

    /// Resolved bytes.
    Bytes(Vec<u8>),

    /// Resolved link.
    Link(Cid),

    /// Promise pending the `ok` branch.
    WaitOk(Cid),

    /// Promise pending the `err` branch.
    WaitErr(Cid),

    /// Promise pending either branch.
    WaitAny(Cid),

    /// Recursively promised list.
    List(Vec<Promised>),

    /// Recursively promised map.
    Map(BTreeMap<String, Promised>),
}

impl Promised {
    pub fn try_resolve(self) -> Result<Ipld, Pending> {
        match self {
            Promised::WaitOk(cid) => Err(Pending::Ok(cid)),
            Promised::WaitErr(cid) => Err(Pending::Err(cid)),
            Promised::WaitAny(cid) => Err(Pending::Any(cid)),
            other => other.try_into().map_err(Into::into),
        }
    }

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

    pub fn to_promise_any<T: TryFrom<Ipld>>(
        self,
    ) -> Result<promise::Any<T>, <T as TryFrom<Ipld>>::Error> {
        Ok(match Ipld::try_from(self) {
            Ok(ipld) => promise::Any::Resolved(ipld.try_into()?),
            Err(pending) => match pending {
                Pending::Ok(cid) => promise::Any::PendingOk(cid),
                Pending::Err(cid) => promise::Any::PendingErr(cid),
                Pending::Any(cid) => promise::Any::PendingAny(cid),
            },
        })
    }

    // FIXME return type
    pub fn to_promise_any_string(self) -> Result<promise::Any<String>, ()> {
        match self {
            Promised::String(s) => Ok(promise::Any::Resolved(s)),
            Promised::WaitOk(cid) => Ok(promise::Any::PendingOk(cid)),
            Promised::WaitErr(cid) => Ok(promise::Any::PendingErr(cid)),
            Promised::WaitAny(cid) => Ok(promise::Any::PendingAny(cid)),
            _ => Err(()),
        }
    }
}

impl fmt::Display for Promised {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
        match self {
            Promised::Null => write!(f, "null"),
            Promised::Bool(b) => write!(f, "{}", b),
            Promised::Integer(i) => write!(f, "{}", i),
            Promised::Float(fl) => write!(f, "{}", fl),
            Promised::String(s) => write!(f, "{}", s),
            Promised::Bytes(b) => write!(f, "{:?}", b),
            Promised::Link(cid) => write!(f, "{}", cid),
            Promised::WaitOk(cid) => write!(f, "await/ok: {}", cid),
            Promised::WaitErr(cid) => write!(f, "await/err: {}", cid),
            Promised::WaitAny(cid) => write!(f, "await/*: {}", cid),
            Promised::List(list) => {
                write!(f, "[")?;
                for (i, promised) in list.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}", promised)?;
                }
                write!(f, "]")
            }
            Promised::Map(map) => {
                write!(f, "{{")?;
                for (i, (k, v)) in map.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", k, v)?;
                }
                write!(f, "}}")
            }
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
                if map.len() == 1 {
                    if let Some((k, Ipld::Link(cid))) = map.first_key_value() {
                        return match k.as_str() {
                            "await/ok" => Promised::WaitOk(*cid),
                            "await/err" => Promised::WaitErr(*cid),
                            "await/*" => Promised::WaitAny(*cid),
                            _ => Promised::Map(BTreeMap::from_iter([(
                                k.to_string(),
                                Promised::Link(*cid),
                            )])),
                        };
                    }
                }

                let map = map.into_iter().fold(BTreeMap::new(), |mut acc, (k, v)| {
                    acc.insert(k, v.into());
                    acc
                });

                Promised::Map(map)
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

impl From<promise::Any<Ipld>> for Promised {
    fn from(p_any: promise::Any<Ipld>) -> Promised {
        match p_any {
            promise::Any::Resolved(ipld) => ipld.into(),
            promise::Any::PendingOk(cid) => Promised::WaitOk(cid),
            promise::Any::PendingErr(cid) => Promised::WaitErr(cid),
            promise::Any::PendingAny(cid) => Promised::WaitAny(cid),
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

impl TryFrom<Promised> for url::Newtype {
    type Error = ();

    fn try_from(promised: Promised) -> Result<url::Newtype, Self::Error> {
        match promised {
            Promised::String(s) => Ok(url::Newtype(::url::Url::parse(&s).map_err(|_| ())?)),
            _ => Err(()),
        }
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
                    self.outbound.push(map);

                    for node in btree.values() {
                        self.inbound.push(node);
                    }
                }

                Some(ref list @ Promised::List(ref vector)) => {
                    self.outbound.push(list);

                    for node in vector {
                        self.inbound.push(node);
                    }
                }
                Some(node) => self.outbound.push(node),
            }
        }
    }
}
