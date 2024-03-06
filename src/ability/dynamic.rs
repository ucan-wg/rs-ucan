//! This module is for dynamic abilities, especially for FFI and Wasm support

use super::{
    arguments,
    command::ToCommand,
    parse::{ParseAbility, ParseAbilityError},
};
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use js_sys;

// NOTE the lack of checking functions!

/// A "dynamic" ability with the bare minimum of statics
///
/// <div class="warning">
/// This should be a last resort, and only for e.g. FFI. The Dynamic ability is
/// <em>not recommended</em> for typical Rust usage.
///
/// This is instead meant to be embedded inside of structs that have e.g. FFI bindings to
/// a validation function, such as `js_sys::Function` for JS, `magnus::function!` for Ruby,
/// and so on.
/// </div>
///
/// [`Dynamic`] uses none of the typical ability traits directly. Rather, it must be wrapped
/// in [`Reader`][crate::reader::Reader], which wires up dynamic dispatch for the
/// relevant traits using a configuration struct.
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)] // FIXME serialize / deserilaize?
pub struct Dynamic {
    /// The `cmd` field (hooks into a dynamic version of [`Command`][crate::ability::command::Command])
    pub cmd: String,

    /// Unstructured, named arguments
    ///
    /// The only requirement is that the keys are strings and the values are [`Ipld`]
    pub args: arguments::Named<Ipld>,
}

impl ParseAbility for Dynamic {
    type ArgsErr = ();

    fn try_parse(
        cmd: &str,
        args: arguments::Named<Ipld>,
    ) -> Result<Self, ParseAbilityError<Self::ArgsErr>> {
        Ok(Dynamic {
            cmd: cmd.to_string(),
            args,
        })
    }
}

impl ToCommand for Dynamic {
    fn to_command(&self) -> String {
        self.cmd.clone()
    }
}

impl From<Dynamic> for arguments::Named<Ipld> {
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
                args: arguments::Named(btree),
            })
        } else {
            Err(JsValue::NULL) // FIXME
        }
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
