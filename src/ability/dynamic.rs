//! This module is for dynamic abilities, especially for FFI and Wasm support

use super::{arguments::Arguments, command::ToCommand};
use crate::{
    delegation::Delegatable,
    invocation::Resolvable,
    ipld,
    promise::Promise,
    proof::{
        checkable::Checkable, parentful::Parentful, parentless::Parentless, parents::CheckParents,
        same::CheckSame,
    },
    task::DefaultTrue,
};
use js_sys;
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{
    de::DeserializeOwned, ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer,
};
use std::{collections::BTreeMap, fmt::Debug};
use wasm_bindgen::prelude::*;

// NOTE the lack of checking functions!
// This is meant to be embedded inside of structs that have e.g. FFI bindings to
// a validation function, such as a &js_sys::Function, Ruby magnus::function!, etc etc
#[derive(Clone, PartialEq, Debug)] // FIXME serialize / deserilaize?
pub struct Dynamic {
    pub cmd: String,
    pub args: Arguments,
}

impl From<Dynamic> for Arguments {
    fn from(dynamic: Dynamic) -> Self {
        dynamic.args
    }
}

#[cfg(target_arch = "wasm32")]
impl From<Dynamic> for js_sys::Map {
    fn from(ability: Dynamic) -> Self {
        let args = js_sys::Map::new();
        for (k, v) in ability.args.0 {
            args.set(&k.into(), &ipld::Newtype(v).into());
        }

        let map = js_sys::Map::new();
        map.set(&"args".into(), &js_sys::Object::from(args).into());
        map.set(&"cmd".into(), &ability.cmd.into());
        map
    }
}

#[cfg(target_arch = "wasm32")]
impl TryFrom<js_sys::Map> for Dynamic {
    type Error = JsValue;

    fn try_from(map: js_sys::Map) -> Result<Self, Self::Error> {
        if let (Some(cmd), js_args) = (
            map.get(&("cmd".into())).as_string(),
            &map.get(&("args".into())),
        ) {
            let obj_args = js_sys::Object::try_from(js_args).ok_or(wasm_bindgen::JsValue::NULL)?;
            let keys = js_sys::Object::keys(obj_args);
            let values = js_sys::Object::values(obj_args);

            let mut btree = BTreeMap::new();
            for (k, v) in keys.iter().zip(values) {
                if let Some(k) = k.as_string() {
                    btree.insert(k, ipld::Newtype::try_from(v).expect("FIXME").0);
                } else {
                    return Err(k);
                }
            }

            Ok(Dynamic {
                cmd,
                args: Arguments(btree), // FIXME kill clone
            })
        } else {
            Err(JsValue::NULL) // FIXME
        }
    }
}

impl CheckSame for Dynamic {
    type Error = String; // FIXME better err

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        if self.cmd != proof.cmd {
            return Err("Command mismatch".into());
        }

        self.args.0.iter().try_for_each(|(k, v)| {
            if let Some(proof_v) = proof.args.0.get(k) {
                if v != proof_v {
                    return Err("Arguments mismatch".into());
                }
            } else {
                return Err("Arguments mismatch".into());
            }

            Ok(())
        })
    }
}

// impl<F> Debug for Generic<Arguments, F> {
//     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
//         f.debug_struct("Generic")
//             .field("cmd", &self.cmd)
//             .field("args", &self.args)
//             .field("is_nonce_meaningful", &self.is_nonce_meaningful)
//             .finish()
//     }
// }
//
// pub type Dynamic<F> = Generic<Arguments, F>;
// pub type Promised<F> = Generic<Promise<Arguments>, F>;
//
// impl<Args: Serialize + Clone> Serialize for Generic<Args, ()>
// where
//     Arguments: From<Args>,
// {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         let mut map = serializer.serialize_map(Some(2))?;
//         map.serialize_entry("cmd", &self.cmd)?;
//         map.serialize_entry("args", &Arguments::from(self.args.clone()))?;
//         map.end()
//     }
// }
//
// impl<'de, Args: Deserialize<'de>> Deserialize<'de> for Generic<Args, ()> {
//     fn deserialize<D>(deserializer: D) -> Result<Generic<Args, ()>, D::Error>
//     where
//         D: Deserializer<'de>,
//     {
//         // FIXME
//         todo!()
//         //         let btree = BTreeMap::deserialize(deserializer)?;
//         //         Ok(Generic {
//         //             cmd: btree.get("cmd")?.to_string(),
//         //             args: btree.get("args")?.clone(),
//         //             is_nonce_meaningful: DefaultTrue::default(),
//         //
//         //             same_validator: (),
//         //             parent_validator: (),
//         //             shape_validator: (),
//         //         })
//     }
// }
//
// impl<Args, F> ToCommand for Generic<Args, F> {
//     fn to_command(&self) -> String {
//         self.cmd.clone()
//     }
// }
//
// impl<F> Delegatable for Dynamic<F> {
//     type Builder = Dynamic<F>;
// }
//
// impl<F> Resolvable for Dynamic<F> {
//     type Promised = Promised<F>;
// }
//
// impl<Args: Serialize> From<Generic<Args, ()>> for Ipld {
//     fn from(generic: Generic<Args, ()>) -> Self {
//         generic.into()
//     }
// }
//
// impl<Args: DeserializeOwned> TryFrom<Ipld> for Generic<Args, ()> {
//     type Error = SerdeError;
//
//     fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
//         ipld_serde::from_ipld(ipld)
//     }
// }
//
// impl<Args: Into<Arguments>, F> From<Generic<Args, F>> for Arguments {
//     fn from(generic: Generic<Args, F>) -> Self {
//         generic.args.into()
//     }
// }
//
// impl<F> TryFrom<Promised<F>> for Dynamic<F> {
//     type Error = (); // FIXME
//
//     fn try_from(awaiting: Promised<F>) -> Result<Self, ()> {
//         if let Promise::Resolved(args) = &awaiting.args {
//             Ok(Dynamic {
//                 cmd: awaiting.cmd,
//                 args: args.clone(),
//
//                 same_validator: awaiting.same_validator,
//                 parent_validator: awaiting.parent_validator,
//                 shape_validator: awaiting.shape_validator,
//                 is_nonce_meaningful: awaiting.is_nonce_meaningful,
//             })
//         } else {
//             Err(())
//         }
//     }
// }
//
// impl<F> From<Dynamic<F>> for Promised<F> {
//     fn from(d: Dynamic<F>) -> Self {
//         Promised {
//             cmd: d.cmd,
//             args: Promise::Resolved(d.args),
//
//             same_validator: d.same_validator,
//             parent_validator: d.parent_validator,
//             shape_validator: d.shape_validator,
//             is_nonce_meaningful: d.is_nonce_meaningful,
//         }
//     }
// }
//
// impl<F> Checkable for Dynamic<F>
// where
//     F: Fn(&String, &Arguments) -> Result<(), String>,
// {
//     type Hierarchy = Parentless<Dynamic<F>>; // FIXME I bet we can revover parents
// }
