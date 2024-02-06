use crate::{
    ability::command::Command,
    proof::{
        parentless::NoParents,
        same::{CheckSame, OptionalFieldErr},
    },
};
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use url::Url;

#[cfg(target_arch = "wasm32")]
use crate::{ipld, proof::same::OptionalFieldReason};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

// NOTE no resolved or awaiting variants, because this cannot be executed, and all fields are optional already!

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Builder {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<Url>,
}

impl Command for Builder {
    const COMMAND: &'static str = "crud/*";
}

impl NoParents for Builder {}

impl CheckSame for Builder {
    type Error = OptionalFieldErr;

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        self.uri
            .check_same(&proof.uri)
            .map_err(|err| OptionalFieldErr {
                field: "uri".into(),
                err,
            })
    }
}

impl TryFrom<Ipld> for Builder {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl From<Builder> for Ipld {
    fn from(builder: Builder) -> Self {
        builder.into()
    }
}

// FIXME
#[derive(Debug, Error)]
pub enum E {
    #[error("Some error")]
    SomeErrMsg(String),
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct CrudAny(#[wasm_bindgen(skip)] pub Builder);

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
        Builder::COMMAND.to_string()
    }

    pub fn check_same(&self, proof: &CrudAny) -> Result<(), JsError> {
        self.0.check_same(&proof.0).map_err(|_| {
            OptionalFieldErr {
                field: "uri".into(),
                err: OptionalFieldReason::MissingField,
            }
            .into()
        })
    }
}
