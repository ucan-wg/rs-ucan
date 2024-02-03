use libipld_core::ipld::Ipld;

pub mod cid;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use js_sys::{Array, Map, Uint8Array};

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
                let u8arr = Uint8Array::new(&bs.len().into());
                for (i, b) in bs.iter().enumerate() {
                    u8arr.set_index(i as u32, *b);
                }
                JsValue::from(u8arr)
            }
            Ipld::List(ls) => {
                let arr = Array::new();
                for ipld in ls {
                    arr.push(&JsValue::from(Newtype(ipld)));
                }
                JsValue::from(arr)
            }
            Ipld::Map(m) => {
                let map = Map::new();
                for (k, v) in m {
                    map.set(&JsValue::from(k), &JsValue::from(Newtype(v)));
                }
                JsValue::from(map)
            }
            Ipld::Link(cid) => cid::Newtype::from(cid).into(),
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

                match (key.as_string(), Newtype::try_from(value.clone())) {
                    (Some(k), Ok(v)) => {
                        m.insert(k, v.0);
                    }
                    _ => {
                        acc = Err(());
                    }
                }
            });

            return acc.map(|_| Newtype(Ipld::Map(m)));
        }

        // NOTE *must* come before `is_object` (which is hopefully below)
        if let Ok(nt) = cid::Newtype::try_from(&js_val) {
            return Ok(Newtype(Ipld::Link(nt.into())));
        }

        if js_val.is_object() {
            let mut m = std::collections::BTreeMap::new();
            let mut acc = Ok(());

            //                            This order is correct per the docs
            //                                       vvvvvvvvvv
            Map::from(js_val).for_each(&mut |value, key| {
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
