//! This module is for dynamic abilities, especially for Wasm support

use super::traits::{Ability, Command};
use crate::prove::TryProve;
use libipld_core::ipld::Ipld;
use std::collections::BTreeMap;

use crate::ipld::WrappedIpld;

use js_sys;
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone, PartialEq)]
pub struct Dynamic<'a> {
    pub command: String,
    pub args: BTreeMap<String, Ipld>, // FIXME consider this being just JsValue
    pub validator: &'a js_sys::Function,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DynamicBuilder<'a> {
    pub command: String,
    pub args: Option<BTreeMap<String, Ipld>>,
    pub validator: &'a js_sys::Function,
}

impl<'a> From<Dynamic<'a>> for DynamicBuilder<'a> {
    fn from(dynamic: Dynamic<'a>) -> Self {
        Self {
            command: dynamic.command.clone(),
            args: Some(dynamic.args),
            validator: dynamic.validator,
        }
    }
}

impl<'a> TryFrom<DynamicBuilder<'a>> for Dynamic<'a> {
    type Error = (); // FIXME

    fn try_from(builder: DynamicBuilder) -> Result<Self, ()> {
        if let Some(args) = builder.clone().args {
            Ok(Self {
                command: builder.command.clone(),
                args,
            })
        } else {
            Err(())
        }
    }
}

impl<'a> Command for Dynamic<'a> {
    fn command(&self) -> &'static str {
        self.command
    }
}

impl<'a> Command for DynamicBuilder<'a> {
    fn command(&self) -> &'static str {
        self.command
    }
}

impl<'a> Ability for Dynamic<'a> {
    type Builder = DynamicBuilder<'a>;
}

impl<'a> TryProve<DynamicBuilder<'a>> for DynamicBuilder<'a> {
    type Error = JsError;
    type Proven = DynamicBuilder<'a>; // TODO docs: even if you parse a well-structred type, you MUST return a dynamic builder and continue checking that

    fn try_prove(&'a self, proof: &'a DynamicBuilder) -> Result<&'a Self::Proven, ()> {
        let js_self: JsValue = self.into().into();
        let js_proof: JsValue = proof.into().into();

        self.validator.apply(js_self, js_proof);
    }
}
