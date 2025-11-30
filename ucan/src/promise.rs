//! Distributed promises

use ipld_core::{cid::Cid, ipld::Ipld};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

/// Top-level union of all UCAN Promise options
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Promise<T, E> {
    /// The `ucan/await/ok` promise
    Ok(T),

    /// The `ucan/await/err` promise
    Err(E),

    /// The `ucan/await/ok` promise
    PendingOk(Cid),

    /// The `ucan/await/err` promise
    PendingErr(Cid),

    /// The `ucan/await/*` promise
    PendingAny(Cid),

    /// The `ucan/await` promise
    PendingTagged(Cid),
}

/// A recursive data structure whose leaves may be [`Ipld`] or promises.
///
/// [`Promised`] resolves to regular [`Ipld`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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

impl TryFrom<&Promised> for Ipld {
    type Error = WaitingOn;

    fn try_from(promised: &Promised) -> Result<Self, Self::Error> {
        match promised {
            Promised::Null => Ok(Ipld::Null),
            Promised::Bool(b) => Ok(Ipld::Bool(*b)),
            Promised::Integer(i) => Ok(Ipld::Integer(*i)),
            Promised::Float(f) => Ok(Ipld::Float(*f)),
            Promised::String(s) => Ok(Ipld::String(s.clone())),
            Promised::Bytes(b) => Ok(Ipld::Bytes(b.clone())),
            Promised::Link(c) => Ok(Ipld::Link(*c)),
            Promised::WaitOk(c) => Err(WaitingOn::WaitOk(*c)),
            Promised::WaitErr(c) => Err(WaitingOn::WaitErr(*c)),
            Promised::WaitAny(c) => Err(WaitingOn::WaitAny(*c)),
            Promised::List(l) => {
                let mut resolved = Vec::new();
                for item in l {
                    resolved.push(Ipld::try_from(item)?);
                }
                Ok(Ipld::List(resolved))
            }
            Promised::Map(m) => {
                let mut resolved = BTreeMap::new();
                for (k, v) in m {
                    resolved.insert(k.clone(), Ipld::try_from(v)?);
                }
                Ok(Ipld::Map(resolved))
            }
        }
    }
}

/// Still waiting to resolve a [`Promised`] value.
#[derive(Debug, Clone, Copy, Error)]
pub enum WaitingOn {
    /// Waiting on the `Ok` branch of a promise that is not yet resolved.
    #[error("Waiting on an `ok` promise {0}")]
    WaitOk(Cid),

    /// Waiting on the `Err` branch of a promise that is not yet resolved.
    #[error("Waiting on an `err` promise {0}")]
    WaitErr(Cid),

    /// Waiting on either branch of a promise that is not yet resolved.
    #[error("Waiting on an `any` promise {0}")]
    WaitAny(Cid),
}
