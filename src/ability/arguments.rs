use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use js_sys::{Array, Map, Uint8Array};

#[cfg(target_arch = "wasm32")]
use crate::ipld;

#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct Pair {
    key: String,
    value: ipld::Newtype,
}

// #[wasm_bindgen]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct Arguments(
    // #[wasm_bindgen(skip)] pub BTreeMap<String, Ipld>,
    #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))] pub BTreeMap<String, Ipld>,
);

// #[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
// #[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
// pub struct Arguments1 {
//     //   #[cfg_attr(target_arch = "wasm32", wasm_bindgen(skip))]
//     #[wasm_bindgen(skip)]
//     pub one: Vec<Ipld>,
// }

impl Arguments {
    pub fn from_iter(iterable: impl IntoIterator<Item = (String, Ipld)>) -> Self {
        Arguments(iterable.into_iter().collect())
    }

    pub fn get(&self, key: &str) -> Option<&Ipld> {
        self.0.get(key)
    }

    pub fn insert(&mut self, key: String, value: Ipld) -> Option<Ipld> {
        self.0.insert(key, value)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &Ipld)> {
        self.0.iter()
    }

    pub fn into_iter(self) -> impl Iterator<Item = (String, Ipld)> {
        self.0.into_iter()
    }
}

impl Default for Arguments {
    fn default() -> Self {
        Arguments(BTreeMap::new())
    }
}

impl TryFrom<Ipld> for Arguments {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl From<Arguments> for Ipld {
    fn from(arguments: Arguments) -> Self {
        ipld_serde::to_ipld(arguments).unwrap()
    }
}

// #[cfg(target_arch = "wasm32")]
// impl From<Arguments> for JsValue {
//     fn from(arguments: Arguments) -> Self {
//         arguments
//             .0
//             .iter()
//             .fold(Map::new(), |map, (ref k, v)| {
//                 map.set(
//                     &JsValue::from_str(k),
//                     &JsValue::from(ipld::Newtype(v.clone())),
//                 );
//                 map
//             })
//             .into()
//     }
// }
//
// #[cfg(target_arch = "wasm32")]
// impl TryFrom<JsValue> for Arguments {
//     type Error = (); // FIXME
//
//     fn try_from(js: JsValue) -> Result<Self, Self::Error> {
//         match ipld::Newtype::try_from(js).map(|newtype| newtype.0) {
//             Err(()) => Err(()), // FIXME surface that we can't parse at all
//             Ok(Ipld::Map(map)) => Ok(Arguments(map)),
//             Ok(_wrong_ipld) => Err(()), // FIXME surface that we have the wrong type
//         }
//     }
// }
