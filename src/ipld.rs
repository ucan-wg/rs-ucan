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
pub struct CidWrapper {
    version_code: u64,
    multicodec_code: u64,
    multihash_code: u64,
    hash_bytes: Vec<u8>,
}

#[cfg(target_arch = "wasm32")]
impl From<CidWrapper> for Cid {
    fn from(wrapper: CidWrapper) -> Self {
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
        .expect("the only way to get a CidWrapper is by deconstructing a valid Cid")
    }
}

#[cfg(target_arch = "wasm32")]
impl From<Cid> for CidWrapper {
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
            Ipld::Bytes(bs) => {
                let buffer = Uint8Array::new(&bs.len().into());
                for (index, item) in bs.iter().enumerate() {
                    buffer.set_index(index as u32, *item);
                }
                JsValue::from(buffer)
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
            // FIXME unclear if this is the correct approach, since the object loses
            // a bunch of info (I presume) -- the JsIpld enum above may be better? Or a class?
            Ipld::Link(cid) => CidWrapper::from(cid).into(),
        }
    }
}
