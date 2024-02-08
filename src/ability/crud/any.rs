use crate::{
    ability::command::Command,
    ipld,
    proof::{error::OptionalFieldError, parentless::NoParents, same::CheckSame},
};
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Any {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<Url>,
}

impl Command for Any {
    const COMMAND: &'static str = "crud/*";
}

impl NoParents for Any {}

impl CheckSame for Any {
    type Error = OptionalFieldError;

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        self.uri.check_same(&proof.uri)
    }
}

impl TryFrom<Ipld> for Any {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl From<Any> for Ipld {
    fn from(builder: Any) -> Self {
        builder.into()
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct CrudAny(#[wasm_bindgen(skip)] pub Any);

// FIXME macro this away
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl CrudAny {
    pub fn to_js(self) -> JsValue {
        ipld::Newtype(Ipld::from(self.0)).into()
    }

    pub fn from_js(js_val: JsValue) -> Result<CrudAny, JsError> {
        ipld::Newtype::try_into_jsvalue(js_val).map(CrudAny)
    }

    pub fn command(&self) -> String {
        Any::COMMAND.to_string()
    }

    pub fn check_same(&self, proof: &CrudAny) -> Result<(), JsError> {
        if self.uri.is_some() {
            if self.uri != proof.uri {
                return Err(OptionalFieldError {
                    field: "uri".into(),
                    err: OptionalFieldReason::NotEqual,
                }
                .into());
            }
        }

        Ok(())
    }
}
