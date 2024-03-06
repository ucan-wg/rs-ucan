//! JavaScript bindings for the CRUD abilities.

use super::read;
use wasm_bindgen::prelude::*;

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
