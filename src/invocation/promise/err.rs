use crate::{ability::arguments, ipld::cid};
use libipld_core::{cid::Cid, error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PromiseErr<E> {
    /// The failure state of a promise.
    Rejected(E),

    /// The [`Cid`] that is being waited on to return an `{"err": value}`
    Pending(#[serde(rename = "await/err")] Cid),
}

impl<E> PromiseErr<E> {
    pub fn try_resolve(self) -> Result<E, PromiseErr<E>> {
        match self {
            PromiseErr::Rejected(err) => Ok(err),
            PromiseErr::Pending(_cid) => Err(self),
        }
    }

    pub fn map<X, F>(self, f: F) -> PromiseErr<X>
    where
        F: FnOnce(E) -> X,
    {
        match self {
            PromiseErr::Rejected(err) => PromiseErr::Rejected(f(err)),
            PromiseErr::Pending(cid) => PromiseErr::Pending(cid),
        }
    }
}

impl<E> From<PromiseErr<E>> for Option<E> {
    fn from(p: PromiseErr<E>) -> Option<E> {
        match p {
            PromiseErr::Rejected(err) => Some(err),
            PromiseErr::Pending(_) => None,
        }
    }
}

impl<E> From<PromiseErr<E>> for Ipld {
    fn from(p: PromiseErr<E>) -> Ipld {
        p.into()
    }
}

impl<E: DeserializeOwned> TryFrom<Ipld> for PromiseErr<E> {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<PromiseErr<E>, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl<E: Into<arguments::Named<Ipld>>> From<PromiseErr<E>> for arguments::Named<Ipld>
where
    Ipld: From<E>,
{
    fn from(p: PromiseErr<E>) -> arguments::Named<Ipld> {
        match p {
            PromiseErr::Rejected(err) => err.into(),
            PromiseErr::Pending(cid) => {
                arguments::Named::from_iter([("await/err".into(), Ipld::Link(cid))])
            }
        }
    }
}

impl<E: TryFrom<Ipld>> TryFrom<arguments::Named<Ipld>> for PromiseErr<E> {
    type Error = <E as TryFrom<Ipld>>::Error;

    fn try_from(args: arguments::Named<Ipld>) -> Result<PromiseErr<E>, Self::Error> {
        if let Some(ipld) = args.get("ucan/err") {
            if args.len() == 1 {
                if let Ok(cid::Newtype { cid }) = cid::Newtype::try_from(ipld) {
                    return Ok(PromiseErr::Pending(cid));
                }
            }
        }

        E::try_from(Ipld::from(args)).map(PromiseErr::Rejected)
    }
}
