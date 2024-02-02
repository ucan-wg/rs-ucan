//! This module is for dynamic abilities, especially for FFI and Wasm support

use super::{arguments::Arguments, command::ToCommand};
use crate::{
    delegation::Delegatable,
    invocation::Resolvable,
    promise::Promise,
    proof::{
        checkable::Checkable, parentful::Parentful, parentless::Parentless, parents::CheckParents,
        same::CheckSame,
    },
    task::DefaultTrue,
};
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{
    de::DeserializeOwned, ser::SerializeMap, Deserialize, Deserializer, Serialize, Serializer,
};
use std::{collections::BTreeMap, fmt::Debug};
use wasm_bindgen::prelude::*;

// NOTE the lack of checking functions!
// This is meant to be embedded inside of structs that have e.g. FFI bindings to
// a validation function, such as a &js_sys::Function, Ruby magnus::function!, etc etc
#[derive(Clone, PartialEq)]
pub struct Generic<Args, F> {
    pub cmd: String,
    pub args: Args,
    pub is_nonce_meaningful: DefaultTrue,

    pub same_validator: F,
    pub parent_validator: F, // FIXME needs to be a different types, and fall back to Void
    pub shape_validator: F,  // FIXME needs to be a different type
}

impl<F> Debug for Generic<Arguments, F> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Generic")
            .field("cmd", &self.cmd)
            .field("args", &self.args)
            .field("is_nonce_meaningful", &self.is_nonce_meaningful)
            .finish()
    }
}

pub type Dynamic<F> = Generic<Arguments, F>;
pub type Promised<F> = Generic<Promise<Arguments>, F>;

impl<Args: Serialize + Clone> Serialize for Generic<Args, ()>
where
    Arguments: From<Args>,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry("cmd", &self.cmd)?;
        map.serialize_entry("args", &Arguments::from(self.args.clone()))?;
        map.end()
    }
}

impl<'de, Args: Deserialize<'de>> Deserialize<'de> for Generic<Args, ()> {
    fn deserialize<D>(deserializer: D) -> Result<Generic<Args, ()>, D::Error>
    where
        D: Deserializer<'de>,
    {
        // FIXME
        todo!()
        //         let btree = BTreeMap::deserialize(deserializer)?;
        //         Ok(Generic {
        //             cmd: btree.get("cmd")?.to_string(),
        //             args: btree.get("args")?.clone(),
        //             is_nonce_meaningful: DefaultTrue::default(),
        //
        //             same_validator: (),
        //             parent_validator: (),
        //             shape_validator: (),
        //         })
    }
}

impl<Args, F> ToCommand for Generic<Args, F> {
    fn to_command(&self) -> String {
        self.cmd.clone()
    }
}

impl<F> Delegatable for Dynamic<F> {
    type Builder = Dynamic<F>;
}

impl<F> Resolvable for Dynamic<F> {
    type Promised = Promised<F>;
}

impl<Args: Serialize> From<Generic<Args, ()>> for Ipld {
    fn from(generic: Generic<Args, ()>) -> Self {
        generic.into()
    }
}

impl<Args: DeserializeOwned> TryFrom<Ipld> for Generic<Args, ()> {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl<Args: Into<Arguments>, F> From<Generic<Args, F>> for Arguments {
    fn from(generic: Generic<Args, F>) -> Self {
        generic.args.into()
    }
}

impl<F> TryFrom<Promised<F>> for Dynamic<F> {
    type Error = (); // FIXME

    fn try_from(awaiting: Promised<F>) -> Result<Self, ()> {
        if let Promise::Resolved(args) = &awaiting.args {
            Ok(Dynamic {
                cmd: awaiting.cmd,
                args: args.clone(),

                same_validator: awaiting.same_validator,
                parent_validator: awaiting.parent_validator,
                shape_validator: awaiting.shape_validator,
                is_nonce_meaningful: awaiting.is_nonce_meaningful,
            })
        } else {
            Err(())
        }
    }
}

impl<F> From<Dynamic<F>> for Promised<F> {
    fn from(d: Dynamic<F>) -> Self {
        Promised {
            cmd: d.cmd,
            args: Promise::Resolved(d.args),

            same_validator: d.same_validator,
            parent_validator: d.parent_validator,
            shape_validator: d.shape_validator,
            is_nonce_meaningful: d.is_nonce_meaningful,
        }
    }
}

impl<F> Checkable for Dynamic<F>
where
    F: Fn(&String, &Arguments) -> Result<(), String>,
{
    type Hierarchy = Parentless<Dynamic<F>>; // FIXME I bet we can revover parents
}

// FIXME Actually, we shoudl go back to wrapping?
// impl<F> CheckSame for Dynamic<F>
// where
//     F: Fn(&String, &Arguments) -> Result<(), String>,
// {
//     type Error = String;
//
//     fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
//         let chain_checker = &self.same_validator;
//         let shape_checker = &self.same_validator;
//
//         shape_checker(&proof.cmd, &proof.args)?;
//         chain_checker(&proof.cmd, &proof.args)
//     }
// }

// #[wasm_bindgen(module = "./ability")]
// extern "C" {
//     type JsAbility;
//
//     // FIXME wrap in func that checks the jsval or better: converts form Ipld
//     #[wasm_bindgen(constructor)]
//     fn new(cmd: String, args: BTreeMap<String, JsValue>) -> JsAbility;
//
//     #[wasm_bindgen(method, getter)]
//     fn command(this: &JsAbility) -> String;
//
//     #[wasm_bindgen(method, getter)]
//     fn arguments(this: &JsAbility) -> Arguments;
//
//     #[wasm_bindgen(method, getter)]
//     fn is_nonce_meaningful(this: &JsAbility) -> bool;
//
//     // e.g. reject extra fields
//     #[wasm_bindgen(method)]
//     fn validate_shape(this: &JsAbility) -> bool;
//
//     // FIXME camels to snakes
//     #[wasm_bindgen(method)]
//     fn check_same(this: &JsAbility, proof: &JsAbility) -> Result<(), String>;
//
//     fn check_parents(th.......)
// }
