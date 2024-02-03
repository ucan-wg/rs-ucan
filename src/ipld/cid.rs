use libipld_core::cid::Cid;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_derive::TryFromJsValue;

// FIXME better name
#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(target_arch = "wasm32", derive(TryFromJsValue))]
#[cfg_attr(target_arch = "wasm32", wasm_bindgen)]
pub struct Newtype {
    cid: Cid,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    /// This is here because the TryFromJsValue derivation macro
    /// doesn't automatically support `Option<T>`.
    ///
    /// [https://docs.rs/wasm-bindgen-derive/0.2.1/wasm_bindgen_derive/#optional-arguments]
    #[wasm_bindgen(typescript_type = "Newtype | undefined")]
    pub type OptionNewtype;
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl Newtype {
    pub fn from_string(cid_string: String) -> Result<Newtype, JsValue> {
        Newtype::try_from(cid_string).map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    pub fn to_string(&self) -> String {
        self.cid.to_string()
    }
}

#[cfg(target_arch = "wasm32")]
impl TryFrom<String> for Newtype {
    type Error = <Cid as TryFrom<String>>::Error;

    fn try_from(cid_string: String) -> Result<Self, Self::Error> {
        Cid::try_from(cid_string).map(Into::into)
    }
}

#[cfg(target_arch = "wasm32")]
impl From<Newtype> for Cid {
    fn from(wrapper: Newtype) -> Self {
        wrapper.cid
    }
}

#[cfg(target_arch = "wasm32")]
impl From<Cid> for Newtype {
    fn from(cid: Cid) -> Self {
        Self { cid }
    }
}
