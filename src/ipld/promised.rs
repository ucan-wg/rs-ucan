use crate::invocation::promise::{Promise, PromiseAny, PromiseErr, PromiseOk, Resolves};
use libipld_core::{cid::Cid, ipld::Ipld};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// A promise to recursively resolve to an [`Ipld`] value.
///
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PromisedIpld {
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
    List(Vec<PromisedIpld>),

    /// [`Ipld::Map`], but where the values are [`PromiseIpld`].
    Map(BTreeMap<String, PromisedIpld>),

    /// Lifted [`Ipld::Link`]
    Link(Cid),

    /// The `await/ok` promise
    PromiseOk(Cid),

    /// The `await/err` promise
    PromiseErr(Cid),

    /// The `await/*` promise
    PromiseAny(Cid),
}

impl PromisedIpld {
    pub fn is_resolved(&self) -> bool {
        match self {
            PromisedIpld::Null => true,
            PromisedIpld::Bool(_) => true,
            PromisedIpld::Integer(_) => true,
            PromisedIpld::Float(_) => true,
            PromisedIpld::String(_) => true,
            PromisedIpld::Bytes(_) => true,
            PromisedIpld::List(list) => list.iter().all(PromisedIpld::is_resolved),
            PromisedIpld::Map(map) => map.values().all(PromisedIpld::is_resolved),
            PromisedIpld::Link(_) => true,
            PromisedIpld::PromiseOk(_) => false,
            PromisedIpld::PromiseErr(_) => false,
            PromisedIpld::PromiseAny(_) => false,
        }
    }

    pub fn is_pending(&self) -> bool {
        !self.is_resolved()
    }
}

impl From<Ipld> for PromisedIpld {
    fn from(ipld: Ipld) -> Self {
        match ipld {
            Ipld::Null => PromisedIpld::Null,
            Ipld::Bool(b) => PromisedIpld::Bool(b),
            Ipld::Integer(i) => PromisedIpld::Integer(i),
            Ipld::Float(f) => PromisedIpld::Float(f),
            Ipld::String(s) => PromisedIpld::String(s),
            Ipld::Bytes(b) => PromisedIpld::Bytes(b),
            Ipld::List(list) => {
                PromisedIpld::List(list.into_iter().map(PromisedIpld::from).collect())
            }
            Ipld::Map(map) => PromisedIpld::Map(
                map.into_iter()
                    .map(|(k, v)| (k, PromisedIpld::from(v)))
                    .collect(),
            ),
            Ipld::Link(cid) => PromisedIpld::Link(cid),
        }
    }
}

impl From<PromiseOk<Ipld>> for PromisedIpld {
    fn from(promise: PromiseOk<Ipld>) -> Self {
        match promise {
            PromiseOk::Fulfilled(ipld) => ipld.into(),
            PromiseOk::Pending(cid) => PromisedIpld::PromiseOk(cid),
        }
    }
}

impl From<PromiseErr<Ipld>> for PromisedIpld {
    fn from(promise: PromiseErr<Ipld>) -> Self {
        match promise {
            PromiseErr::Rejected(ipld) => ipld.into(),
            PromiseErr::Pending(cid) => PromisedIpld::PromiseErr(cid),
        }
    }
}

impl From<PromiseAny<Ipld, Ipld>> for PromisedIpld {
    fn from(promise: PromiseAny<Ipld, Ipld>) -> Self {
        match promise {
            PromiseAny::Fulfilled(ipld) => ipld.into(),
            PromiseAny::Rejected(ipld) => ipld.into(),
            PromiseAny::Pending(cid) => PromisedIpld::PromiseAny(cid),
        }
    }
}

impl From<Resolves<Ipld>> for PromisedIpld {
    fn from(resolves: Resolves<Ipld>) -> Self {
        match resolves {
            Resolves::Ok(p_ok) => p_ok.into(),
            Resolves::Err(p_err) => p_err.into(),
        }
    }
}

impl From<Promise<Ipld, Ipld>> for PromisedIpld {
    fn from(promise: Promise<Ipld, Ipld>) -> Self {
        match promise {
            Promise::Ok(p_ok) => p_ok.into(),
            Promise::Err(p_err) => p_err.into(),
            Promise::Any(p_any) => p_any.into(),
        }
    }
}

impl TryFrom<PromisedIpld> for Ipld {
    type Error = PromisedIpld;

    fn try_from(p: PromisedIpld) -> Result<Self, Self::Error> {
        match p {
            PromisedIpld::Null => Ok(Ipld::Null),
            PromisedIpld::Bool(b) => Ok(Ipld::Bool(b)),
            PromisedIpld::Integer(i) => Ok(Ipld::Integer(i)),
            PromisedIpld::Float(f) => Ok(Ipld::Float(f)),
            PromisedIpld::String(s) => Ok(Ipld::String(s)),
            PromisedIpld::Bytes(b) => Ok(Ipld::Bytes(b)),
            PromisedIpld::List(ref list) => {
                let result: Result<Vec<Ipld>, ()> = list.iter().try_fold(vec![], |mut acc, x| {
                    let ipld = Ipld::try_from(x.clone()).map_err(|_| ())?;
                    acc.push(ipld);
                    Ok(acc)
                });

                Ok(Ipld::List(result.map_err(|_| p.clone())?))
            }
            PromisedIpld::Map(ref map) => {
                let map: Result<BTreeMap<String, Ipld>, ()> =
                    map.into_iter()
                        .try_fold(BTreeMap::new(), |mut acc, (k, v)| {
                            // FIXME non-tail recursion, and maybe even repeated clones
                            let ipld = Ipld::try_from(v.clone()).map_err(|_| ())?;
                            acc.insert(k.clone(), ipld);
                            Ok(acc)
                        });

                Ok(Ipld::Map(map.map_err(|_| p)?))
            }
            PromisedIpld::Link(cid) => Ok(Ipld::Link(cid)),
            PromisedIpld::PromiseOk(_cid) => Err(p),
            PromisedIpld::PromiseErr(_cid) => Err(p),
            PromisedIpld::PromiseAny(_cid) => Err(p),
        }
    }
}
