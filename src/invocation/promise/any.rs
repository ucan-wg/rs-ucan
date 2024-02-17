use super::{err::PromiseErr, ok::PromiseOk};
use crate::{ability::arguments, ipld::cid};
use libipld_core::{cid::Cid, error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{
    de::{Deserializer, Error, MapAccess, Visitor},
    Deserialize, Serialize,
};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(untagged)]
pub enum PromiseAny<T, E> {
    /// The fulfilled (resolved) value.
    Fulfilled(#[serde(rename = "ucan/ok")] T),

    /// The failure state of a promise.
    Rejected(#[serde(rename = "ucan/err")] E),

    /// A deferred value and its associated [`Selector`].
    ///
    /// The [`Selector`] will resolve a branch from the [`Receipt`][crate::receipt::Receipt]
    /// and substitute into the value.
    Pending(#[serde(rename = "await/*")] Cid),
}

impl<'de, T: Deserialize<'de>, E: Deserialize<'de>> Deserialize<'de> for PromiseAny<T, E> {
    fn deserialize<D>(deserializer: D) -> Result<PromiseAny<T, E>, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct PromiseAnyVisitor<T, E>(std::marker::PhantomData<(T, E)>);

        impl<'de, T: Deserialize<'de>, E: Deserialize<'de>> Visitor<'de> for PromiseAnyVisitor<T, E> {
            type Value = PromiseAny<T, E>;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a promise")
            }

            fn visit_map<M>(self, mut map: M) -> Result<PromiseAny<T, E>, M::Error>
            where
                M: MapAccess<'de>,
            {
                if map.size_hint() != Some(1) {
                    return Err(serde::de::Error::custom("expected a single key"));
                }

                let key = map
                    .next_key::<String>()?
                    .ok_or(Error::invalid_length(0, &"expected exactly 1 key"))?;

                match key.as_str() {
                    "ucan/ok" => {
                        let val = map.next_value()?;
                        return Ok(PromiseAny::Fulfilled(val));
                    }

                    "ucan/err" => {
                        let err = map.next_value()?;
                        return Ok(PromiseAny::Rejected(err));
                    }

                    "await/*" => {
                        let cid = map.next_value()?;
                        return Ok(PromiseAny::Pending(cid));
                    }

                    _ => return Err(serde::de::Error::custom("expected a valid PromiseAny")),
                }
            }
        }

        deserializer.deserialize_map(PromiseAnyVisitor(std::marker::PhantomData))
    }
}

impl<T, E> PromiseAny<T, E> {
    pub fn map<U, F>(self, f: F) -> PromiseAny<U, E>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            PromiseAny::Fulfilled(val) => PromiseAny::Fulfilled(f(val)),
            PromiseAny::Rejected(err) => PromiseAny::Rejected(err),
            PromiseAny::Pending(cid) => PromiseAny::Pending(cid),
        }
    }

    pub fn map_err<X, F>(self, f: F) -> PromiseAny<T, X>
    where
        F: FnOnce(E) -> X,
    {
        match self {
            PromiseAny::Fulfilled(val) => PromiseAny::Fulfilled(val),
            PromiseAny::Rejected(err) => PromiseAny::Rejected(f(err)),
            PromiseAny::Pending(cid) => PromiseAny::Pending(cid),
        }
    }
}

impl<T, E> From<PromiseAny<T, E>> for Ipld {
    fn from(p: PromiseAny<T, E>) -> Ipld {
        p.into()
    }
}

impl<T, E> TryFrom<Ipld> for PromiseAny<T, E>
where
    T: for<'de> Deserialize<'de>,
    E: for<'de> Deserialize<'de>,
{
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<PromiseAny<T, E>, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl<T: Into<Ipld>, E: Into<Ipld>> From<PromiseAny<T, E>> for arguments::Named<Ipld> {
    fn from(p: PromiseAny<T, E>) -> arguments::Named<Ipld> {
        match p {
            PromiseAny::Fulfilled(val) => {
                arguments::Named::from_iter([("ucan/ok".into(), val.into())])
            }
            PromiseAny::Rejected(err) => {
                arguments::Named::from_iter([("ucan/err".into(), err.into())])
            }
            PromiseAny::Pending(cid) => {
                arguments::Named::from_iter([("await/*".into(), cid.into())])
            }
        }
    }
}

impl<T: for<'a> TryFrom<&'a Ipld>, E: for<'a> TryFrom<&'a Ipld>> TryFrom<arguments::Named<Ipld>>
    for PromiseAny<T, E>
{
    type Error = (); // FIXME

    fn try_from(args: arguments::Named<Ipld>) -> Result<PromiseAny<T, E>, Self::Error> {
        if args.len() != 1 {
            return Err(());
        }

        if let Some(ipld) = args.get("ucan/ok") {
            return Ok(PromiseAny::Fulfilled(ipld.try_into().map_err(|_| ())?));
        }

        if let Some(ipld) = args.get("ucan/err") {
            return Ok(PromiseAny::Rejected(ipld.try_into().map_err(|_| ())?));
        }

        if let Some(ipld) = args.get("await/*") {
            return match cid::Newtype::try_from(ipld) {
                Ok(nt) => Ok(PromiseAny::Pending(nt.cid)),
                Err(_) => Err(()),
            };
        }

        Err(())
    }
}

impl<T, E> From<PromiseOk<T>> for PromiseAny<T, E> {
    fn from(p_ok: PromiseOk<T>) -> PromiseAny<T, E> {
        match p_ok {
            PromiseOk::Fulfilled(val) => PromiseAny::Fulfilled(val),
            PromiseOk::Pending(cid) => PromiseAny::Pending(cid),
        }
    }
}

impl<T, E> From<PromiseErr<E>> for PromiseAny<T, E> {
    fn from(p_err: PromiseErr<E>) -> PromiseAny<T, E> {
        match p_err {
            PromiseErr::Rejected(err) => PromiseAny::Rejected(err),
            PromiseErr::Pending(cid) => PromiseAny::Pending(cid),
        }
    }
}

impl<T, E> TryFrom<PromiseAny<T, E>> for PromiseOk<T> {
    type Error = (); // FIXME

    fn try_from(p_any: PromiseAny<T, E>) -> Result<PromiseOk<T>, Self::Error> {
        match p_any {
            PromiseAny::Fulfilled(val) => Ok(PromiseOk::Fulfilled(val)),
            PromiseAny::Rejected(_err) => Err(()),
            PromiseAny::Pending(cid) => Ok(PromiseOk::Pending(cid)),
        }
    }
}

impl<T, E> TryFrom<PromiseAny<T, E>> for PromiseErr<E> {
    type Error = (); // FIXME

    fn try_from(p_any: PromiseAny<T, E>) -> Result<PromiseErr<E>, Self::Error> {
        match p_any {
            PromiseAny::Fulfilled(_val) => Err(()),
            PromiseAny::Rejected(err) => Ok(PromiseErr::Rejected(err)),
            PromiseAny::Pending(cid) => Ok(PromiseErr::Pending(cid)),
        }
    }
}
