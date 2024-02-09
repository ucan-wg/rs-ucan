use libipld_core::{cid::Cid, ipld::Ipld};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub enum Enriched<T> {
    /// Lifted [`Ipld::Null`]
    Null,

    /// Lifted [`Ipld::Bool`]
    Bool(bool),

    /// Lifted [`Ipld::Integer`]
    Integer(i128),

    /// Lifted [`Ipld::Float`]
    Float(f64),

    /// Lifted [`Ipld::String`]
    String(String),

    /// Lifted [`Ipld::Bytes`] (byte array)
    Bytes(Vec<u8>),

    /// [`Ipld::List`], but where the values are [`PromiseIpld`].
    List(Vec<T>),

    /// [`Ipld::Map`], but where the values are [`PromiseIpld`].
    Map(BTreeMap<String, T>),

    /// Lifted [`Ipld::Link`]
    Link(Cid),
}

impl<T: From<Ipld>> From<Ipld> for Enriched<T> {
    fn from(ipld: Ipld) -> Self {
        match ipld {
            Ipld::Null => Enriched::Null,
            Ipld::Bool(b) => Enriched::Bool(b),
            Ipld::Integer(i) => Enriched::Integer(i),
            Ipld::Float(f) => Enriched::Float(f),
            Ipld::String(s) => Enriched::String(s),
            Ipld::Bytes(b) => Enriched::Bytes(b),
            Ipld::List(l) => Enriched::List(l.into_iter().map(From::from).collect()),
            Ipld::Map(m) => Enriched::Map(m.into_iter().map(|(k, v)| (k, From::from(v))).collect()),
            Ipld::Link(c) => Enriched::Link(c),
        }
    }
}

impl<T: Clone + TryInto<Ipld>> TryFrom<Enriched<T>> for Ipld {
    type Error = Enriched<T>;

    fn try_from(enriched: Enriched<T>) -> Result<Ipld, Self::Error> {
        match enriched {
            Enriched::List(ref vec) => {
                let result: Result<Vec<Ipld>, ()> = vec.iter().try_fold(vec![], |mut acc, x| {
                    let resolved = x.clone().try_into().map_err(|_| ())?;
                    acc.push(resolved);
                    Ok(acc)
                });

                match result {
                    Ok(vec) => Ok(vec.into()),
                    Err(()) => Err(enriched),
                }
            }
            Enriched::Map(ref btree) => {
                let result: Result<BTreeMap<String, Ipld>, ()> =
                    btree.iter().try_fold(BTreeMap::new(), |mut acc, (k, v)| {
                        let resolved = v.clone().try_into().map_err(|_| ())?;
                        acc.insert(k.clone(), resolved);
                        Ok(acc)
                    });

                match result {
                    Ok(vec) => Ok(vec.into()),
                    Err(()) => Err(enriched),
                }
            }
            Enriched::Null => Ok(Ipld::Null),
            Enriched::Bool(b) => Ok(b.into()),
            Enriched::Integer(i) => Ok(i.into()),
            Enriched::Float(f) => Ok(f.into()),
            Enriched::String(s) => Ok(s.into()),
            Enriched::Bytes(b) => Ok(b.into()),
            Enriched::Link(l) => Ok(l.into()),
        }
    }
}
