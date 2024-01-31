use crate::capsule::Capsule;
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq)]
pub struct Envelope<T: Capsule> {
    pub sig: Signature,
    pub payload: T,
}

// FIXME consider kicking Batch down the road for spec reasons?
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Signature {
    One(Vec<u8>),
    Batch {
        sig: Vec<u8>,
        merkle_proof: Vec<Vec<u8>>,
    },
}

// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(transparent)]
// pub struct StaticVec<T> {
//     pub slice: Box<[T]>,
// }
//
// impl<T> From<Vec<T>> for StaticVec<T> {
//     fn from(vec: Vec<T>) -> Self {
//         Self {
//             slice: vec.into_boxed_slice(),
//         }
//     }
// }
//
// impl<T> From<StaticVec<T>> for Vec<T> {
//     fn from(vec: StaticVec<T>) -> Vec<T> {
//         vec.slice.into()
//     }
// }
//
// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[serde(transparent)]
// pub struct StaticString {
//     string: Box<str>,
// }
//
// impl From<String> for StaticString {
//     fn from(string: String) -> Self {
//         Self {
//             string: string.into_boxed_str(),
//         }
//     }
// }
//
// impl<'a> From<&'a str> for StaticString {
//     fn from(s: &'a str) -> Self {
//         Self { string: s.into() }
//     }
// }
//
// impl<'a> From<&'a StaticString> for &'a str {
//     fn from(s: &'a StaticString) -> &'a str {
//         &s.string
//     }
// }
//
// impl From<StaticString> for String {
//     fn from(s: StaticString) -> String {
//         s.string.into()
//     }
// }

impl From<&Signature> for Ipld {
    fn from(sig: &Signature) -> Self {
        match sig {
            Signature::One(sig) => sig.clone().into(),
            Signature::Batch { sig, merkle_proof } => {
                let mut map = BTreeMap::new();
                let proof: Vec<Ipld> = merkle_proof.into_iter().map(|p| p.clone().into()).collect();
                map.insert("sig".into(), sig.clone().into());
                map.insert("prf".into(), proof.into());
                map.into()
            }
        }
    }
}

impl From<Signature> for Ipld {
    fn from(sig: Signature) -> Self {
        match sig {
            Signature::One(sig) => sig.into(),
            Signature::Batch { sig, merkle_proof } => {
                let mut map = BTreeMap::new();
                let proof: Vec<Ipld> = merkle_proof.into_iter().map(|p| p.into()).collect();
                map.insert("sig".into(), sig.into());
                map.insert("prf".into(), proof.into());
                map.into()
            }
        }
    }
}

// FIXME Store or BTreeMap? Also eliminate that Clone constraint
impl<T: Capsule + Into<Ipld> + Clone> From<&Envelope<T>> for Ipld {
    fn from(Envelope { sig, payload }: &Envelope<T>) -> Self {
        let mut inner = BTreeMap::new();
        inner.insert(T::TAG.into(), payload.clone().into()); // FIXME should be a link

        let mut map = BTreeMap::new();
        map.insert("sig".into(), sig.into());
        map.insert("pld".into(), Ipld::Map(inner));

        Ipld::Map(map)
    }
}

impl<T: Capsule + Into<Ipld> + Clone> From<Envelope<T>> for Ipld {
    fn from(Envelope { sig, payload }: Envelope<T>) -> Self {
        let mut inner = BTreeMap::new();
        inner.insert(T::TAG.into(), payload.clone().into()); // FIXME should be a link

        let mut map = BTreeMap::new();
        map.insert("sig".into(), sig.into());
        map.insert("pld".into(), Ipld::Map(inner));

        Ipld::Map(map)
    }
}
