use libipld_core::ipld::Ipld;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::JsValue;

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
    fn from(wrapped: WrappedIpld) -> Self {
        match wrapped.0 {
            Ipld::Null => JsValue::Null,
            Ipld::Bool(b) => JsValue::from(b),
            Ipld::Integer(i) => JsValue::from(i),
            Ipld::Float(f) => JsValue::from_f64(f),
            Ipld::String(s) => JsValue::from_str(&s),
            Ipld::Bytes(b) => JsValue::from(b),
            Ipld::List(l) => {
                let mut vec = Vec::new();
                for ipld in l {
                    vec.push(JsValue::from(ipld));
                }
                JsValue::from(vec)
            }
            Ipld::Map(m) => {
                let mut map = JsValue::new();
                for (k, v) in m {
                    map.set(&k, JsValue::from(v));
                }
                map
            }
            Ipld::Link(l) => JsValue::from(Link::from(l)),
        }
    }
}
