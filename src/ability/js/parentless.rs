use crate::{
    ability::{arguments::Arguments, command::ToCommand, dynamic},
    ipld,
    proof::{checkable::Checkable, parentless::Parentless, parents::CheckParents, same::CheckSame},
};
use js_sys::{Function, JsString, Map, Object, Reflect};
use libipld_core::ipld::Ipld;
use std::collections::BTreeMap;
use wasm_bindgen::{prelude::*, JsValue};

// NOTE NOTE NOTE: the strategy is: "you (JS) hand us the cfg" AKA strategy,
// and we (Rust) wire it up and run it for you
// NOTE becuase of the above, no need to export JsWithParents to JS
// FIXME rename
type JsWithoutParents = dynamic::Configured<Config1>;

// FIXME rename ability? abilityconfig? leave as is?
// #[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone, PartialEq)]
#[wasm_bindgen]
pub struct Config1 {
    // #[wasm_bindgen(skip)]
    command: String,
    is_nonce_meaningful: bool,

    // #[wasm_bindgen(skip)]
    validate_shape: Function,

    //#[wasm_bindgen(skip)]
    check_same: Function,
}

// FIXME represent promises (for Promised) and options (for builder)

#[wasm_bindgen]
impl Config1 {
    // FIXME object args as an option
    #[wasm_bindgen(constructor)]
    pub fn new(
        command: String,
        is_nonce_meaningful: bool,
        validate_shape: Function,
        check_same: Function,
    ) -> Config1 {
        Config1 {
            command,
            is_nonce_meaningful,
            validate_shape,
            check_same,
        }
    }
}

impl From<JsWithoutParents> for dynamic::Dynamic {
    fn from(js: JsWithoutParents) -> Self {
        dynamic::Dynamic {
            cmd: js.config.command,
            args: js.arguments,
        }
    }
}

// FIXME while this can totally be done by converting to the dynamic carrier type, this seems more straightforward?
impl CheckSame for JsWithoutParents {
    type Error = JsValue;

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        let this = wasm_bindgen::JsValue::NULL;
        self.config
            .check_same
            .call2(
                &this,
                &self.arguments.clone().into(),
                &Arguments::from(proof.clone()).into(),
            )
            .map(|_| ())
    }
}

impl ToCommand for Config1 {
    fn to_command(&self) -> String {
        self.command.clone()
    }
}

impl Checkable for JsWithoutParents {
    type Hierarchy = Parentless<JsWithoutParents>;
}
