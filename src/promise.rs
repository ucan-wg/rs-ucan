use cid::Cid;
use libipld_core::{ipld::Ipld, serde as ipld_serde};
use serde_derive::{Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Deferrable<T> {
    Resolved(T),
    Await(Promise),
}

impl<T> Deferrable<T> {
    pub fn try_extract(self) -> Result<T, Self> {
        match self {
            Deferrable::Resolved(t) => Ok(t),
            Deferrable::Await(promise) => Err(Deferrable::Await(promise)),
        }
    }
}

impl<T> From<T> for Deferrable<T> {
    fn from(t: T) -> Self {
        Deferrable::Resolved(t)
    }
}

/// A [`Promise`] is a way to defer the presence of a value to the result of some [`Invocation`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)] // FIXME check that this is right, also
pub enum Promise {
    PromiseAny {
        #[serde(rename = "ucan/*")] // FIXME test to make sure that this is right?
        any: Cid,
    },
    PromiseOk {
        #[serde(rename = "ucan/ok")]
        ok: Cid,
    },
    PromiseErr {
        #[serde(rename = "ucan/err")]
        err: Cid,
    },
}

impl TryFrom<Ipld> for Promise {
    type Error = ();

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld).map_err(|_| ())
    }
}
