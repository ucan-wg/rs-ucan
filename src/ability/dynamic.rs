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
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)] // FIXME serialize / deserilaize?
pub struct Dynamic {
    pub cmd: String,
    pub args: Arguments,
}

// NOTE plug this into Configured<T> like: Configured<Resolved<Dynamic>>
pub struct Builder<T>(pub T);
pub struct Promised<T>(pub T);

impl<T: Into<Arguments>> From<Builder<T>> for Arguments {
    fn from(builder: Builder<T>) -> Self {
        builder.0.into()
    }
}

impl<T> From<Configured<T>> for Builder<Configured<T>> {
    fn from(configured: Configured<T>) -> Self {
        Builder(configured)
    }
}

impl<T: ToCommand> From<Builder<Configured<T>>> for Configured<T> {
    fn from(builder: Builder<Configured<T>>) -> Self {
        builder.0
    }
}

impl<T: Into<Arguments>> From<Promised<T>> for Arguments {
    fn from(promised: Promised<T>) -> Self {
        promised.0.into()
    }
}

impl<T> From<Configured<T>> for Promised<Configured<T>> {
    fn from(configured: Configured<T>) -> Self {
        Promised(configured)
    }
}

impl<T: ToCommand> From<Promised<Configured<T>>> for Configured<T> {
    fn from(promised: Promised<Configured<T>>) -> Self {
        promised.0
    }
}

// NOTE to self: this is helpful as a common container to lift various FFI into
#[derive(Clone, PartialEq, Debug)]
pub struct Configured<T> {
    pub arguments: Arguments,
    pub config: T,
}

impl<T: ToCommand> Delegatable for Configured<T> {
    type Builder = Builder<Configured<T>>;
}

impl<T: ToCommand> Resolvable for Configured<T> {
    type Promised = Promised<Configured<T>>;
}

impl<T: ToCommand> ToCommand for Configured<T> {
    fn to_command(&self) -> String {
        self.config.to_command()
    }
}

impl<T: CheckSame> CheckSame for Configured<T> {
    type Error = T::Error;

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        self.config.check_same(&proof.config)
    }
}

impl<T: CheckParents> CheckParents for Configured<T> {
    type Parents = Dynamic;
    type ParentError = T::ParentError;

    fn check_parents(&self, parent: &Dynamic) -> Result<(), Self::ParentError> {
        self.check_parents(parent)
    }
}

impl<T> From<Configured<T>> for Arguments {
    fn from(reader: Configured<T>) -> Self {
        reader.arguments
    }
}

impl From<Dynamic> for Arguments {
    fn from(dynamic: Dynamic) -> Self {
        dynamic.args
    }
}

impl<T: Checkable> Checkable for Configured<T> {
    type Hierarchy = T::Hierarchy;
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

impl From<Dynamic> for Ipld {
    fn from(dynamic: Dynamic) -> Self {
        dynamic.into()
    }
}

impl TryFrom<Ipld> for Dynamic {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}
