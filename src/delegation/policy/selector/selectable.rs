use super::error::SelectorErrorReason;
use crate::ipld;
use libipld_core::ipld::Ipld;
use std::collections::BTreeMap;

pub trait Selectable: Sized {
    fn try_select(ipld: Ipld) -> Result<Self, SelectorErrorReason>;
}

impl Selectable for Ipld {
    fn try_select(ipld: Ipld) -> Result<Ipld, SelectorErrorReason> {
        Ok(ipld)
    }
}

impl Selectable for ipld::Newtype {
    fn try_select(ipld: Ipld) -> Result<ipld::Newtype, SelectorErrorReason> {
        Ok(ipld::Newtype(ipld))
    }
}

impl Selectable for ipld::Number {
    fn try_select(ipld: Ipld) -> Result<ipld::Number, SelectorErrorReason> {
        match ipld {
            Ipld::Integer(i) => Ok(ipld::Number::Integer(i)),
            Ipld::Float(f) => Ok(ipld::Number::Float(f)),
            _ => Err(SelectorErrorReason::NotANumber),
        }
    }
}

impl Selectable for String {
    fn try_select(ipld: Ipld) -> Result<Self, SelectorErrorReason> {
        match ipld {
            Ipld::String(s) => Ok(s),
            _ => Err(SelectorErrorReason::NotAString),
        }
    }
}

impl Selectable for ipld::Collection {
    fn try_select(ipld: Ipld) -> Result<ipld::Collection, SelectorErrorReason> {
        match ipld {
            Ipld::List(xs) => Ok(ipld::Collection::Array(xs.into_iter().try_fold(
                vec![],
                |mut acc, v| {
                    acc.push(Selectable::try_select(v)?);
                    Ok(acc)
                },
            )?)),
            Ipld::Map(xs) => Ok(ipld::Collection::Map(xs.into_iter().try_fold(
                BTreeMap::new(),
                |mut map, (k, v)| {
                    let value = Selectable::try_select(v)?;
                    map.insert(k, value);
                    Ok(map)
                },
            )?)),
            _ => Err(SelectorErrorReason::NotACollection),
        }
    }
}
