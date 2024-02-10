//! JavaScript bindings for the CRUD abilities.

use super::{read, Any};
use wasm_bindgen::prelude::*;

/// DOCS?
#[wasm_bindgen]
pub struct CrudAny(#[wasm_bindgen(skip)] pub Any);

// FIXME macro this away
#[wasm_bindgen]
impl CrudAny {
    pub fn into_js(self) -> JsValue {
        ipld::Newtype(Ipld::from(self.0)).into()
    }

    pub fn try_from_js(js: JsValue) -> Result<CrudAny, JsError> {
        ipld::Newtype::try_from_js(js).map(CrudAny)
    }

    pub fn to_command(&self) -> String {
        self.to_command()
    }

    pub fn check_same(&self, proof: &CrudAny) -> Result<(), JsError> {
        if self.path.is_some() {
            if self.path != proof.path {
                return Err(OptionalFieldError {
                    field: "path".into(),
                    err: OptionalFieldReason::NotEqual,
                }
                .into());
            }
        }

        Ok(())
    }
}

#[wasm_bindgen]
pub struct CrudRead(#[wasm_bindgen(skip)] pub read::Ready);

#[wasm_bindgen]
impl CrudRead {
    pub fn to_jsvalue(self) -> JsValue {
        ipld::Newtype(Ipld::from(self.0)).into()
    }

    pub fn from_jsvalue(js_val: JsValue) -> Result<CrudRead, JsError> {
        ipld::Newtype::try_into_jsvalue(js_val).map(CrudRead)
    }

    pub fn to_command(&self) -> String {
        Read::to_command()
    }

    pub fn check_same(&self, proof: &CrudRead) -> Result<(), JsError> {
        self.0.check_same(&proof.0).map_err(Into::into)
    }

    // FIXME more than any
    pub fn check_parent(&self, proof: &CrudAny) -> Result<(), JsError> {
        self.0.check_parent(&proof.0).map_err(Into::into)
    }
}

// FIXME needs bindings
#[wasm_bindgen]
pub struct CrudReadPromise(#[wasm_bindgen(skip)] pub read::Promised);
