use crate::ability::arguments::Arguments;
use cid::Cid;
use libipld_core::{ipld::Ipld, serde as ipld_serde};
use serde_derive::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Debug};

// FIXME move under invocation?

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Promise<T> {
    Resolved(T),
    Waiting(Selector),
}

impl<T> Promise<T> {
    pub fn map<U, F>(self, f: F) -> Promise<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Promise::Resolved(t) => Promise::Resolved(f(t)),
            Promise::Waiting(selector) => Promise::Waiting(selector),
        }
    }
}

impl<T: Into<Arguments>> From<Promise<T>> for Arguments {
    fn from(promise: Promise<T>) -> Self {
        match promise {
            Promise::Resolved(t) => t.into(),
            Promise::Waiting(selector) => selector.into(),
        }
    }
}

impl<T> Promise<T> {
    pub fn try_extract(self) -> Result<T, Self> {
        match self {
            Promise::Resolved(t) => Ok(t),
            Promise::Waiting(promise) => Err(Promise::Waiting(promise)),
        }
    }
}

impl<T> From<T> for Promise<T> {
    fn from(t: T) -> Self {
        Promise::Resolved(t)
    }
}

impl<T> From<Promise<T>> for Ipld
where
    T: Into<Ipld>,
{
    fn from(promise: Promise<T>) -> Self {
        match promise {
            Promise::Resolved(t) => t.into(),
            Promise::Waiting(selector) => selector.into(),
        }
    }
}

/// A [`Promise`] is a way to defer the presence of a value to the result of some [`Invocation`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)] // FIXME check that this is right, also
pub enum Selector {
    Any {
        #[serde(rename = "ucan/*")] // FIXME test to make sure that this is right?
        any: Cid,
    },
    Ok {
        #[serde(rename = "ucan/ok")]
        ok: Cid,
    },
    Err {
        #[serde(rename = "ucan/err")]
        err: Cid,
    },
}

impl From<Selector> for Ipld {
    fn from(selector: Selector) -> Self {
        selector.into()
    }
}

impl TryFrom<Ipld> for Selector {
    type Error = ();

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld).map_err(|_| ())
    }
}

impl From<Selector> for Arguments {
    fn from(selector: Selector) -> Self {
        let mut btree = BTreeMap::new();

        match selector {
            Selector::Any { any } => {
                btree.insert("ucan/*".into(), any.into());
            }
            Selector::Ok { ok } => {
                btree.insert("ucan/ok".into(), ok.into());
            }
            Selector::Err { err } => {
                btree.insert("ucan/err".into(), err.into());
            }
        }

        Arguments(btree)
    }
}
