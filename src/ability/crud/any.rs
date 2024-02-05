use crate::{
    ability::command::Command,
    proof::{
        parentless::NoParents,
        same::{CheckSame, OptionalFieldErr},
    },
};
use serde::{Deserialize, Serialize};
use url::Url;

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
        self.uri.check_same(&proof.uri)
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct CrudAny(#[wasm_bindgen(skip)] pub Builder);

// FIXME macro this away
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl CrudAny {
    pub fn command(&self) -> String {
        Builder::COMMAND.to_string()
    }

    pub fn check_same(&self, proof: &CrudAny) -> Result<(), JsValue> {
        self.0
            .check_same(&proof.0)
            .map_err(|err| JsValue::from_str(&format!("{:?}", err)))
    }
}
