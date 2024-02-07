use crate::ability::arguments;
use libipld_core::{cid::Cid, ipld::Ipld, serde as ipld_serde};
use serde_derive::{Deserialize, Serialize};
use std::{collections::BTreeMap, fmt::Debug};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// FIXME move under invocation?

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Promise<T> {
    /// The fulfilled (resolved) value.
    Fulfilled(T),

    // Rejected(E)
    //
    /// A deferred value and its associated [`Selector`].
    ///
    /// The [`Selector`] will resolve a branch from the [`Receipt`][crate::receipt::Receipt]
    /// and substitute into the value.
    Pending(Selector), // FIXME shodu there be FulfilledOk and FulfilledErr? If so, why cover al branches here?
}

#[cfg(target_arch = "wasm32")]
#[derive(Clone, Debug, PartialEq, Eq)]
#[wasm_bindgen]
pub enum UcanPromiseStatus {
    Fulfilled,
    Pending,
}

// FIXME no way to make this consistent, because of C enums ruining Rust convetions, right?
// FIXME consider wrapping in a trait
#[cfg(target_arch = "wasm32")]
#[derive(Clone, Debug, PartialEq)]
#[wasm_bindgen]
pub struct UcanPromise {
    status: UcanPromiseStatus,
    selector: Option<Selector>,
    value: Option<JsValue>,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(getter_with_clone)]
pub struct IncoherentPromise(pub UcanPromise);

#[cfg(target_arch = "wasm32")]
impl TryFrom<UcanPromise> for Promise<JsValue> {
    type Error = IncoherentPromise;

    fn try_from(js: UcanPromise) -> Result<Self, Self::Error> {
        match js.status {
            UcanPromiseStatus::Fulfilled => {
                if let Some(val) = &js.value {
                    return Ok(Promise::Fulfilled(val.clone()));
                }
            }
            UcanPromiseStatus::Pending => {
                if let Some(selector) = &js.selector {
                    return Ok(Promise::Pending(selector.clone()));
                }
            }
        }

        Err(IncoherentPromise(js))
    }
}

#[cfg(target_arch = "wasm32")]
impl<T: Into<JsValue>> From<Promise<T>> for UcanPromise {
    fn from(promise: Promise<T>) -> Self {
        match promise {
            Promise::Fulfilled(val) => UcanPromise {
                status: UcanPromiseStatus::Fulfilled,
                selector: None,
                value: Some(val.into()),
            },
            Promise::Pending(cid) => UcanPromise {
                status: UcanPromiseStatus::Pending,
                selector: Some(cid),
                value: None,
            },
        }
    }
}

impl<T> Promise<T> {
    pub fn map<U, F>(self, f: F) -> Promise<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            Promise::Fulfilled(t) => Promise::Fulfilled(f(t)),
            Promise::Pending(selector) => Promise::Pending(selector),
        }
    }
}

impl<T: Into<arguments::Named>> From<Promise<T>> for arguments::Named {
    fn from(promise: Promise<T>) -> Self {
        match promise {
            Promise::Fulfilled(t) => t.into(),
            Promise::Pending(selector) => selector.into(),
        }
    }
}

impl<T> From<T> for Promise<T> {
    fn from(value: T) -> Self {
        Promise::Fulfilled(value)
    }
}

impl<T> Promise<T> {
    pub fn try_resolve(self) -> Result<T, Selector> {
        match self {
            Promise::Fulfilled(t) => Ok(t),
            Promise::Pending(selector) => Err(selector),
        }
    }
}

impl<T> From<Promise<T>> for Ipld
where
    T: Into<Ipld>,
{
    fn from(promise: Promise<T>) -> Self {
        match promise {
            Promise::Fulfilled(t) => t.into(),
            Promise::Pending(selector) => selector.into(),
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

impl From<Selector> for arguments::Named {
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

        arguments::Named(btree)
    }
}
