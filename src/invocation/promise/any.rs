use crate::ipld;
use super::pending::Pending;
use enum_as_inner::EnumAsInner;
use libipld_core::cid::Cid;
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// FIXME
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EnumAsInner)]
pub enum Any<T> {
    /// The `ucan/await/ok` promise
    Resolved(T),

    /// The `ucan/await/ok` promise
    PendingOk(Cid),

    /// The `ucan/await/err` promise
    PendingErr(Cid),

    /// The `ucan/await/*` promise
    PendingAny(Cid),
}

impl<T> Any<T> {
    pub fn try_resolve(self) -> Result<T, Any<T>> {
        match self {
            Any::Resolved(value) => Ok(value),
            _ => Err(self),
        }
    }

    pub fn from_ipld(ipld: Ipld) -> Self
    where
        T: From<Ipld>,
    {
        match ipld {
            Ipld::Map(ref map) => {
                if let Some(Ipld::Link(cid)) = map.get("ucan/await/ok") {
                    return Any::PendingOk(cid.clone());
                }

                if let Some(Ipld::Link(cid)) = map.get("ucan/await/err") {
                    return Any::PendingErr(cid.clone());
                }

                if let Some(Ipld::Link(cid)) = map.get("ucan/await/*") {
                    return Any::PendingAny(cid.clone());
                }

                Any::Resolved(ipld.into())
            }
            other => Any::Resolved(other.into()),
        }
    }

    pub fn to_promised_ipld(self) -> ipld::Promised
    where
        T: Into<ipld::Promised>,
    {
        match self {
            Any::Resolved(value) => value.into(),
            Any::PendingOk(cid) => ipld::Promised::WaitOk(cid),
            Any::PendingErr(cid) => ipld::Promised::WaitErr(cid),
            Any::PendingAny(cid) => ipld::Promised::WaitAny(cid),
        }
    }
}

impl<T> From<Pending> for Any<T> {
    fn from(pending: Pending) -> Any<T> {
        match pending {
            Pending::Ok(cid) => Any::PendingOk(cid),
            Pending::Err(cid) => Any::PendingErr(cid),
            Pending::Any(cid) => Any::PendingAny(cid),
        }
    }
}

impl<T: Into<Ipld>> From<Any<T>> for Ipld {
    fn from(promise: Any<T>) -> Ipld {
        match promise {
            Any::Resolved(val) => val.into(),
            Any::PendingOk(cid) => Ipld::Map(BTreeMap::from_iter([(
                "ucan/await/ok".to_string(),
                cid.into(),
            )])),
            Any::PendingErr(cid) => Ipld::Map(BTreeMap::from_iter([(
                "ucan/await/err".to_string(),
                cid.into(),
            )])),
            Any::PendingAny(cid) => Ipld::Map(BTreeMap::from_iter([(
                "ucan/await/*".to_string(),
                cid.into(),
            )])),
        }
    }
}

impl<T: TryFrom<ipld::Promised>> TryFrom<ipld::Promised> for Any<T> {
    type Error = <T as TryFrom<ipld::Promised>>::Error;

    fn try_from(promised: ipld::Promised) -> Result<Any<T>, Self::Error> {
        match promised {
            ipld::Promised::WaitOk(cid) => Ok(Any::PendingOk(cid)),
            ipld::Promised::WaitErr(cid) => Ok(Any::PendingErr(cid)),
            ipld::Promised::WaitAny(cid) => Ok(Any::PendingAny(cid)),
            other => Ok(Any::Resolved(T::try_from(other)?)),
        }
    }
}
