//FIXME rename core
use super::{
    collection::Collection,
    selector::{error::SelectorErrorReason, SelectorError},
};
use crate::ipld;
use libipld_core::ipld::Ipld;
use std::collections::BTreeMap;

pub trait Resolve<T> {
    fn resolve(self, ctx: &Ipld) -> Result<T, SelectorError>;
}

impl Resolve<Ipld> for Ipld {
    fn resolve(self, _ctx: &Ipld) -> Result<Ipld, SelectorError> {
        Ok(self)
    }
}

impl Resolve<ipld::Newtype> for ipld::Newtype {
    fn resolve(self, _ctx: &Ipld) -> Result<ipld::Newtype, SelectorError> {
        Ok(self)
    }
}

impl Resolve<ipld::Number> for ipld::Number {
    fn resolve(self, _ctx: &Ipld) -> Result<ipld::Number, SelectorError> {
        Ok(self)
    }
}

impl Resolve<Collection> for Collection {
    fn resolve(self, _ctx: &Ipld) -> Result<Collection, SelectorError> {
        Ok(self)
    }
}

impl Resolve<String> for String {
    fn resolve(self, _ctx: &Ipld) -> Result<String, SelectorError> {
        Ok(self)
    }
}

pub trait TryFromIpld: Sized {
    fn try_from_ipld(ipld: Ipld) -> Result<Self, SelectorErrorReason>;
}

impl TryFromIpld for Ipld {
    fn try_from_ipld(ipld: Ipld) -> Result<Ipld, SelectorErrorReason> {
        Ok(ipld)
    }
}

impl TryFromIpld for ipld::Newtype {
    fn try_from_ipld(ipld: Ipld) -> Result<ipld::Newtype, SelectorErrorReason> {
        Ok(ipld::Newtype(ipld))
    }
}

impl TryFromIpld for ipld::Number {
    fn try_from_ipld(ipld: Ipld) -> Result<ipld::Number, SelectorErrorReason> {
        match ipld {
            Ipld::Integer(i) => Ok(ipld::Number::Integer(i)),
            Ipld::Float(f) => Ok(ipld::Number::Float(f)),
            _ => Err(SelectorErrorReason::NotANumber),
        }
    }
}

impl TryFromIpld for String {
    fn try_from_ipld(ipld: Ipld) -> Result<Self, SelectorErrorReason> {
        match ipld {
            Ipld::String(s) => Ok(s),
            _ => Err(SelectorErrorReason::NotAString),
        }
    }
}

impl TryFromIpld for Collection {
    fn try_from_ipld(ipld: Ipld) -> Result<Collection, SelectorErrorReason> {
        match ipld {
            Ipld::List(xs) => Ok(Collection::Array(xs.into_iter().try_fold(
                vec![],
                |mut acc, v| {
                    acc.push(TryFromIpld::try_from_ipld(v)?);
                    Ok(acc)
                },
            )?)),
            Ipld::Map(xs) => Ok(Collection::Map(xs.into_iter().try_fold(
                BTreeMap::new(),
                |mut map, (k, v)| {
                    let value = TryFromIpld::try_from_ipld(v)?;
                    map.insert(k, value);
                    Ok(map)
                },
            )?)),
            _ => Err(SelectorErrorReason::NotACollection),
        }
    }
}
