use crate::{ability::arguments, ipld::cid};
use libipld_core::{cid::Cid, error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::fmt::Debug;

#[cfg(feature = "test_utils")]
use proptest::prelude::*;

/// A promise that only selects the `{"ok": value}` branch of a result.
///
/// On resolution, the value is unwrapped from the `{"ok": value}`,
/// leaving just the `value` (much like [`Result::unwrap`]).
///
/// FIXME exmaple
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged, deny_unknown_fields)]
pub enum PromiseOk<T> {
    /// The fulfilled (resolved) value.
    Fulfilled(T),

    /// The [`Cid`] that is being waited on to return an `{"ok": value}`
    Pending(#[serde(rename = "await/ok")] Cid),
}

// FIXME move try_resolve to a trait, give a blanket impl for prims, and tag them
impl<T> PromiseOk<T> {
    pub fn try_resolve(self) -> Result<T, PromiseOk<T>> {
        match self {
            PromiseOk::Fulfilled(value) => Ok(value),
            PromiseOk::Pending(_cid) => Err(self),
        }
    }

    pub fn map<U, F>(self, f: F) -> PromiseOk<U>
    where
        F: FnOnce(T) -> U,
    {
        match self {
            PromiseOk::Fulfilled(val) => PromiseOk::Fulfilled(f(val)),
            PromiseOk::Pending(cid) => PromiseOk::Pending(cid),
        }
    }
}

impl<T> From<PromiseOk<T>> for Option<T> {
    fn from(p: PromiseOk<T>) -> Option<T> {
        match p {
            PromiseOk::Fulfilled(value) => Some(value),
            PromiseOk::Pending(_) => None,
        }
    }
}

impl<T> From<PromiseOk<T>> for Ipld {
    fn from(p: PromiseOk<T>) -> Ipld {
        p.into()
    }
}

impl<T: DeserializeOwned> TryFrom<Ipld> for PromiseOk<T> {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<PromiseOk<T>, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl<T: Into<arguments::Named<Ipld>>> From<PromiseOk<T>> for arguments::Named<Ipld>
where
    Ipld: From<T>,
{
    fn from(p: PromiseOk<T>) -> arguments::Named<Ipld> {
        match p {
            PromiseOk::Fulfilled(val) => val.into(),
            PromiseOk::Pending(cid) => {
                arguments::Named::from_iter([("await/ok".into(), Ipld::Link(cid))])
            }
        }
    }
}

impl<T: TryFrom<Ipld>> TryFrom<arguments::Named<Ipld>> for PromiseOk<T> {
    type Error = <T as TryFrom<Ipld>>::Error;

    fn try_from(args: arguments::Named<Ipld>) -> Result<PromiseOk<T>, Self::Error> {
        if let Some(ipld) = args.get("ucan/ok") {
            if args.len() == 1 {
                if let Ok(cid::Newtype { cid }) = cid::Newtype::try_from(ipld) {
                    return Ok(PromiseOk::Pending(cid));
                }
            }
        }

        T::try_from(Ipld::from(args)).map(PromiseOk::Fulfilled)
    }
}

#[cfg(feature = "test_utils")]
impl<T: Arbitrary + Debug + 'static> Arbitrary for PromiseOk<T>
where
    T::Strategy: 'static,
    T::Parameters: 'static,
{
    type Parameters = T::Parameters;
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(t_args: Self::Parameters) -> Self::Strategy {
        prop_oneof![
            T::arbitrary_with(t_args).prop_map(PromiseOk::Fulfilled),
            cid::Newtype::arbitrary().prop_map(|nt| PromiseOk::Pending(nt.cid)),
        ]
        .boxed()
    }
}
