//! This module is for dynamic abilities, especially for FFI and Wasm support

use super::{arguments::Arguments, command::ToCommand};
use crate::{delegation::Delegatable, invocation::Resolvable, promise::Promise};
use serde_derive::{Deserialize, Serialize};
use std::fmt::Debug;

// FIXME move commented-out module?
// use js_sys;
// use wasm_bindgen::prelude::*;
// type JsDynamic = Dynamic<&'a js_sys::Function>;
// type JsBuilder = Builder<&'a js_sys::Function>;
// type JsPromised = Promised<&'a js_sys::Function>;
// FIXME move these fiels to a wrapper struct in a different module
//     #[serde(skip_serializing)]
//     pub chain_validator: Pred,
//     #[serde(skip_serializing)]
//     pub shape_validator: Pred,
//     #[serde(skip_serializing)]
//     pub serialize_nonce: DefaultTrue,

// NOTE the lack of checking functions!
// This is meant to be embedded inside of structs that have e.g. FFI bindings to
// a validation function, such as a &js_sys::Function, Ruby magnus::function!, etc etc
#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Generic<Args> {
    pub cmd: String,
    pub args: Args,
}

pub type Dynamic = Generic<Arguments>;
pub type Promised = Generic<Promise<Arguments>>;

impl<Args> ToCommand for Generic<Args> {
    fn to_command(&self) -> String {
        self.cmd.clone()
    }
}

impl Delegatable for Dynamic {
    type Builder = Dynamic;
}

impl Resolvable for Dynamic {
    type Promised = Dynamic;
}

impl From<Dynamic> for Arguments {
    fn from(dynamic: Dynamic) -> Self {
        dynamic.args
    }
}

impl TryFrom<Promised> for Dynamic {
    type Error = (); // FIXME

    fn try_from(awaiting: Promised) -> Result<Self, ()> {
        if let Promise::Resolved(args) = &awaiting.args {
            Ok(Dynamic {
                cmd: awaiting.cmd,
                args: args.clone(),
            })
        } else {
            Err(())
        }
    }
}
