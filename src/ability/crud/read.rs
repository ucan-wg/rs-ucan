use super::any;
use crate::{
    ability::command::Command,
    proof::{checkable::Checkable, parentful::Parentful, parents::CheckParents, same::CheckSame},
};
use libipld_core::{ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use url::Url;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// Read is its own builder
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Read {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<Url>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<BTreeMap<String, Ipld>>, // FIXME rename Argumenst to get the traits?
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct Js(#[wasm_bindgen(skip)] pub Read);

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl Js {
    // FIXME
    pub fn check_same(&self, proof: &Js) -> Result<(), JsValue> {
        self.0
            .check_same(&proof.0)
            .map_err(|err| JsValue::from_str(&format!("{:?}", err)))
    }
}

impl Command for Read {
    const COMMAND: &'static str = "crud/read";
}

impl From<Read> for Ipld {
    fn from(read: Read) -> Self {
        read.into()
    }
}

impl TryFrom<Ipld> for Read {
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld).map_err(|_| ())
    }
}

impl Checkable for Read {
    type Hierarchy = Parentful<Read>;
}

impl CheckSame for Read {
    type Error = ();
    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl CheckParents for Read {
    type Parents = any::Builder;
    type ParentError = ();
    fn check_parents(&self, other: &Self::Parents) -> Result<(), Self::ParentError> {
        Ok(())
    }
}
