use crate::{
    ability::{arguments, command::Command, crud::any::CrudAny},
    proof::{checkable::Checkable, parentful::Parentful, parents::CheckParents, same::CheckSame},
};
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use crate::ipld;

// Read is its own builder
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Read {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<Url>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<arguments::Named>,
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
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

// FIXME
#[derive(Debug, Error)]
pub enum E {
    #[error("Some error")]
    SomeErrMsg(String),
}

impl Checkable for Read {
    type Hierarchy = Parentful<Read>;
}

impl CheckSame for Read {
    type Error = E;
    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        if let Some(uri) = &self.uri {
            if uri != proof.uri.as_ref().unwrap() {
                return Err(E::SomeErrMsg("".into()));
            }
        }

        if let Some(args) = &self.args {
            if let Some(proof_args) = &proof.args {
                for (k, v) in args.iter() {
                    if proof_args.get(k) != Some(v) {
                        return Err(E::SomeErrMsg("".into()));
                    }
                }
            }
        }

        Ok(())
    }
}

impl CheckParents for Read {
    type Parents = super::Any;
    type ParentError = E;

    fn check_parent(&self, _other: &Self::Parents) -> Result<(), Self::ParentError> {
        Ok(()) // FIXME
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct CrudRead(#[wasm_bindgen(skip)] pub Read);

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl CrudRead {
    pub fn to_jsvalue(self) -> JsValue {
        ipld::Newtype(Ipld::from(self.0)).into()
    }

    pub fn from_jsvalue(js_val: JsValue) -> Result<CrudRead, JsError> {
        ipld::Newtype::try_into_jsvalue(js_val).map(CrudRead)
    }

    pub fn command(&self) -> String {
        Read::COMMAND.to_string()
    }

    pub fn check_same(&self, proof: &CrudRead) -> Result<(), JsError> {
        self.0.check_same(&proof.0).map_err(Into::into)
    }

    pub fn check_parent(&self, proof: &CrudAny) -> Result<(), JsError> {
        self.0.check_parent(&proof.0).map_err(Into::into)
    }
}
