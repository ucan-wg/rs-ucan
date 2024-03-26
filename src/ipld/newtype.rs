use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use js_sys::{Array, Map, Object, Uint8Array};

#[cfg(feature = "test_utils")]
use proptest::prelude::*;

#[cfg(feature = "test_utils")]
use super::cid;

#[cfg(target_arch = "wasm32")]
use super::cid;

/// A newtype wrapper around [`Ipld`] that has additional trait implementations.
///
/// Usage is very simple: wrap a [`Newtype`] to gain access to additional traits and methods.
///
/// ```rust
/// # use libipld_core::ipld::Ipld;
/// # use ucan::ipld;
/// #
/// let ipld = Ipld::String("hello".into());
/// let wrapped = ipld::Newtype(ipld.clone());
/// // wrapped.some_trait_method();
/// ```
///
/// Unwrap a [`Newtype`] to use any interfaces that expect plain [`Ipld`].
///
/// ```
/// # use libipld_core::ipld::Ipld;
/// # use ucan::ipld;
/// #
/// # let ipld = Ipld::String("hello".into());
/// # let wrapped = ipld::Newtype(ipld.clone());
/// #
/// assert_eq!(wrapped.0, ipld);
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct Newtype(pub Ipld);

impl From<Ipld> for Newtype {
    fn from(ipld: Ipld) -> Self {
        Self(ipld)
    }
}

impl From<i128> for Newtype {
    fn from(i: i128) -> Self {
        Newtype(Ipld::Integer(i))
    }
}

impl From<f64> for Newtype {
    fn from(f: f64) -> Self {
        Newtype(Ipld::Float(f))
    }
}

impl From<&str> for Newtype {
    fn from(s: &str) -> Self {
        Newtype(Ipld::String(s.to_string()))
    }
}

impl From<String> for Newtype {
    fn from(s: String) -> Self {
        Newtype(Ipld::String(s))
    }
}

impl TryFrom<Newtype> for String {
    type Error = ();

    fn try_from(nt: Newtype) -> Result<String, ()> {
        match nt.0 {
            Ipld::String(s) => Ok(s),
            _ => Err(()),
        }
    }
}

impl TryFrom<Newtype> for i128 {
    type Error = ();

    fn try_from(nt: Newtype) -> Result<i128, ()> {
        match nt.0 {
            Ipld::Integer(i) => Ok(i),
            _ => Err(()),
        }
    }
}

impl From<Newtype> for Ipld {
    fn from(wrapped: Newtype) -> Self {
        wrapped.0
    }
}

impl From<PathBuf> for Newtype {
    fn from(path: PathBuf) -> Self {
        Newtype(Ipld::String(path.to_string_lossy().to_string()))
    }
}

impl TryFrom<Newtype> for PathBuf {
    type Error = NotAString;

    fn try_from(wrapped: Newtype) -> Result<Self, Self::Error> {
        match wrapped.0 {
            Ipld::String(s) => Ok(PathBuf::from(s)),
            ipld => Err(NotAString(ipld)),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Error)]
#[error("Ipld variant is not a string")]
pub struct NotAString(pub Ipld);

#[cfg(target_arch = "wasm32")]
impl Newtype {
    pub fn try_from_js<T: TryFrom<Ipld>>(js_val: JsValue) -> Result<T, JsError>
    where
        JsError: From<<T as TryFrom<Ipld>>::Error>,
    {
        match Newtype::try_from(js_val) {
            Err(_err) => Err(JsError::new("can't convert")),
            Ok(nt) => nt.0.try_into().map_err(JsError::from),
        }
    }
}

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
        if let Ok(nt) = cid::Newtype::try_from_js_value(&js_val) {
            return Ok(Newtype(Ipld::Link(nt.into())));
        }

        if js_val.is_object() {
            let obj = Object::from(js_val);
            let mut m = std::collections::BTreeMap::new();
            let mut acc = Ok(());

            Object::entries(&obj).for_each(&mut |js_val, _, _| {
                if acc.is_err() {
                    return;
                }

                // By definition this must be the array [value, key], in that order
                let arr = Array::from(&js_val);

                match (arr.get(0).as_string(), Newtype::try_from(arr.get(1))) {
                    (Some(k), Ok(v)) => {
                        m.insert(k, v.0);
                    }
                    // FIXME more specific errors
                    _ => {
                        acc = Err(());
                    }
                }
            });

            return acc.map(|_| Newtype(Ipld::Map(m)));
        }

        // NOTE fails on `undefined` and `function`

        Err(())
    }
}

#[cfg(feature = "test_utils")]
impl Arbitrary for Newtype {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        let leaf = prop_oneof![
            Just(Ipld::Null),
            any::<bool>().prop_map(Ipld::Bool),
            any::<Vec<u8>>().prop_map(Ipld::Bytes),
            any::<i128>().prop_map(move |i| {
                Ipld::Integer((i % (2 ^ 53)).into()) // NOTE Because DAG-JSON
            }),
            any::<f64>().prop_map(Ipld::Float),
            ".*".prop_map(Ipld::String),
            any::<cid::Newtype>().prop_map(|newtype_cid| { Ipld::Link(newtype_cid.cid) })
        ];

        let coll = leaf.clone().prop_recursive(16, 1024, 128, |inner| {
            prop_oneof![
                prop::collection::vec(inner.clone(), 0..128).prop_map(Ipld::List),
                prop::collection::btree_map(".*", inner, 0..128).prop_map(Ipld::Map),
            ]
        });

        prop_oneof![
            1 => leaf,
            9 => coll
        ]
        .prop_map(Newtype)
        .boxed()
    }
}
