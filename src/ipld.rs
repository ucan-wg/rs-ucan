use libipld_core::ipld::Ipld;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use libipld_core::cid::Cid;

#[cfg(target_arch = "wasm32")]
use libipld_core::multihash::MultihashGeneric;

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

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub struct NewCid {
    // FIXME Link?
    version_code: u64,
    multicodec_code: u64,
    multihash_code: u64,
    hash_bytes: Vec<u8>,
}

// FIXME better name
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl NewCid {
    pub fn from_string(cid_string: String) -> Result<Link, JsValue> {
        NewCid::try_from(cid_string).map_err(|e| JsValue::from_str(&format!("{}", e)))
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
        Cid::new(
            wrapper.version_code.try_into().expect(
                "must be a valid Cid::Version because it was deconstructed from a valid one",
            ),
            wrapper
                .multicodec_code
                .try_into()
                .expect("must be a valid Multicodec because it was deconstructed from a valid one"),
            MultihashGeneric::<64>::wrap(wrapper.multihash_code, wrapper.hash_bytes.as_ref())
                .expect("a valid Multihash because it was deconstructed from a valid one")
                .into(),
        )
        .expect("the only way to get a Link is by deconstructing a valid Cid")
    }
}

#[cfg(target_arch = "wasm32")]
impl From<Cid> for NewCid {
    fn from(cid: Cid) -> Self {
        Self {
            version_code: cid.version().into(),
            multicodec_code: cid.codec().into(),
            multihash_code: cid.hash().code(),
            hash_bytes: cid.hash().digest().to_vec(),
        }
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
            Ipld::Bytes(bs) => Bytes::from(bs).into(),
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
            let mut ls = Vec::with_capacity(arr.length() as usize);
            for a in arr.iter() {
                ls.push(Newtype::try_from(a)?.into());
            }
            return Ok(Newtype(Ipld::List(ls)));
        }

        if let Some(bytes) = js_val.dyn_ref::<Uint8Array>() {
            let arr = Uint8Array::new(&bytes.raw.len().into());

            for (index, item) in bytes.raw.iter().enumerate() {
                arr.set_index(index as u32, *item);
            }

            return Ok(Newtype(Ipld::Bytes(arr)));
        }

        if let Some(map) = js_val.dyn_ref::<Map>() {
            let mut m = std::collections::BTreeMap::new();

            for iter_item in map.entries() {
                match iter_item {
                    Err(_) => return Err(()),
                    Ok(k) => match k.as_string() {
                        None => return Err(()),
                        Some(s) =>  {
                            m.insert(s, Newtype::try_from(v)?.0);
                        },
                    }
                }
            }

            return Ok(Newtype(Ipld::Map(m)));
        }

        // NOTE *must* come before `is_object` (which is hopefully below)
        if let Some(link) = js_val.dyn_ref::<NewCid>() {
            return Ok(Newtype(Ipld::Link(link.into().clone())));
        }

        if js_val.is_object() {
            let mut m = std::collections::BTreeMap::new();
            for (k, v) in js_val {
                m.insert(k.as_string.ok_or(())?, Newtype::try_from(v)?);
            }

            return Ok(Newtype(Ipld::Map(m)));
        }

        //         // FIXME hmmmm
        //         if let Some(undefined) = js_val.is_undefined() {
        //             return Self(Ipld::Null);
        //         }

        Err(())
    }
}
