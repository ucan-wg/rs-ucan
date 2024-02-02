use libipld_core::ipld::Ipld;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use libipld_core::cid::Cid;

#[cfg(target_arch = "wasm32")]
use libipld_core::multihash::MultihashGeneric;

#[cfg(target_arch = "wasm32")]
use js_sys::{Array, Map, Uint8Array};

#[cfg(target_arch = "wasm32")]
use wasm_bindgen_derive::TryFromJsValue;

pub struct Newtype(pub Ipld);

impl From<Ipld> for Newtype {
    fn from(ipld: Ipld) -> Self {
        Self(ipld)
    }
}

impl From<Newtype> for Ipld {
    fn from(wrapped: Newtype) -> Self {
        wrapped.0
    }
}

// FIXME better name
#[cfg(target_arch = "wasm32")]
#[derive(TryFromJsValue, Debug, PartialEq, Eq, Clone)]
#[wasm_bindgen]
pub struct NewCid {
    cid: Cid,
}

#[wasm_bindgen]
extern "C" {
    /// This is here because the TryFromJsValue derivation macro
    /// doesn't automatically support `Option<T>`.
    ///
    /// [https://docs.rs/wasm-bindgen-derive/0.2.1/wasm_bindgen_derive/#optional-arguments]
    #[wasm_bindgen(typescript_type = "NewCid | undefined")]
    pub type OptionNewCid;
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl NewCid {
    pub fn from_string(cid_string: String) -> Result<NewCid, JsValue> {
        NewCid::try_from(cid_string).map_err(|e| JsValue::from_str(&format!("{}", e)))
    }

    pub fn to_string(&self) -> String {
        self.cid.to_string()
    }
}

#[cfg(target_arch = "wasm32")]
impl TryFrom<String> for NewCid {
    type Error = <Cid as TryFrom<String>>::Error;

    fn try_from(cid_string: String) -> Result<Self, Self::Error> {
        Cid::try_from(cid_string).map(Into::into)
    }
}

#[cfg(target_arch = "wasm32")]
impl From<NewCid> for Cid {
    fn from(wrapper: NewCid) -> Self {
        wrapper.cid
    }
}

#[cfg(target_arch = "wasm32")]
impl From<Cid> for NewCid {
    fn from(cid: Cid) -> Self {
        Self { cid }
    }
}

// TODO testme
#[cfg(target_arch = "wasm32")]
impl From<Newtype> for JsValue {
    fn from(wrapped: Newtype) -> Self {
        match wrapped.0 {
            Ipld::Null => JsValue::NULL,
            Ipld::Bool(b) => JsValue::from(b),
            Ipld::Integer(i) => JsValue::from(i),
            Ipld::Float(f) => JsValue::from_f64(f),
            Ipld::String(s) => JsValue::from_str(&s),
            Ipld::Bytes(bs) => {
                let mut arr = js_sys::Uint8Array::new(&bs.len().into());
                for (i, b) in bs.iter().enumerate() {
                    arr.set_index(i as u32, *b);
                }
                arr.into()
            }
            Ipld::List(ls) => {
                let mut arr = Array::new();
                for ipld in ls {
                    arr.push(&JsValue::from(Newtype(ipld)));
                }
                JsValue::from(arr)
            }
            Ipld::Map(m) => {
                let mut map = Map::new();
                for (k, v) in m {
                    map.set(&JsValue::from(k), &JsValue::from(Newtype(v)));
                }
                JsValue::from(map)
            }
            Ipld::Link(cid) => NewCid::from(cid).into(),
        }
    }
}

// TODO testme
#[cfg(target_arch = "wasm32")]
impl TryFrom<JsValue> for Newtype {
    type Error = (); // FIXME

    fn try_from(js_val: JsValue) -> Result<Self, Self::Error> {
        if js_val.is_null() {
            return Ok(Newtype(Ipld::Null));
        }

        if let Some(b) = js_val.as_bool() {
            return Ok(Newtype(Ipld::Bool(b)));
        }

        if let Some(f) = js_val.as_f64() {
            return Ok(Newtype(Ipld::Float(f)));
        }

        if let Some(s) = js_val.as_string() {
            return Ok(Newtype(Ipld::String(s)));
        }

        if let Some(arr) = js_val.dyn_ref::<Array>() {
            let mut list = vec![];
            for x in arr.to_vec().iter() {
                let ipld = Newtype::try_from(x.clone())?.into();
                list.push(ipld);
            }

            return Ok(Newtype(Ipld::List(list)));
        }

        if let Some(arr) = js_val.dyn_ref::<Uint8Array>() {
            let mut v = vec![];
            for item in arr.to_vec().iter() {
                v.push(item.clone());
            }

            return Ok(Newtype(Ipld::Bytes(v)));
        }

        if let Some(map) = js_val.dyn_ref::<Map>() {
            let mut m = std::collections::BTreeMap::new();
            let mut acc = Ok(());

            //      Weird order, but correct per the docs
            //                 vvvvvvvvvv
            map.for_each(&mut |value, key| {
                if acc.is_err() {
                    return;
                }

                match key.as_string() {
                    None => {
                        acc = Err(());
                    }
                    Some(k) => match Newtype::try_from(value.clone()) {
                        Err(_) => {
                            acc = Err(());
                        }
                        Ok(v) => match Newtype::try_from(value) {
                            Err(_) => {
                                acc = Err(());
                            }
                            Ok(v) => {
                                m.insert(k, v.0);
                            }
                        },
                    },
                }
            });

            return acc.map(|_| Newtype(Ipld::Map(m)));
        }

        // NOTE *must* come before `is_object` (which is hopefully below)
        if let Ok(new_cid) = NewCid::try_from(&js_val) {
            return Ok(Newtype(Ipld::Link(new_cid.cid)));
        }

        if js_val.is_object() {
            let mut m = std::collections::BTreeMap::new();
            let mut acc = Ok(());

            //                            This order is correct per the docs
            //                                       vvvvvvvvvv
            js_sys::Map::from(js_val).for_each(&mut |value, key| {
                if acc.is_err() {
                    return;
                }

                match key.as_string() {
                    None => {
                        acc = Err(());
                    }
                    Some(k) => match Newtype::try_from(value) {
                        Err(_) => {
                            acc = Err(());
                        }
                        Ok(v) => {
                            m.insert(k, v.0);
                        }
                    },
                }
            });

            return acc.map(|_| Newtype(Ipld::Map(m)));
        }

        // NOTE fails on `undefined` and `function`

        Err(())
    }
}
