use crate::{
    ability::{arguments::Arguments, command::ToCommand, dynamic},
    ipld,
    proof::{checkable::Checkable, parentless::Parentless, same::CheckSame},
};
use js_sys::{Function, Map, Object, Reflect};
use libipld_core::ipld::Ipld;
use std::collections::BTreeMap;
use wasm_bindgen::{prelude::*, JsValue};

#[derive(Debug, Clone, PartialEq)]
#[wasm_bindgen]
pub struct JsWithoutParents {
    #[wasm_bindgen(skip)]
    pub ability: dynamic::Dynamic,

    #[wasm_bindgen(skip)]
    pub config: Config,
}

// FIXME just inline
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub is_nonce_meaningful: bool,
    pub validate_shape: Function,
    pub check_same: Function,
}

#[wasm_bindgen]
impl JsWithoutParents {
    // FIXME consider using an object with named fields
    // FIXME needs borrows?
    #[wasm_bindgen(constructor)]
    pub fn new(
        cmd: String,
        args: Object,
        is_nonce_meaningful: bool,
        validate_shape: js_sys::Function,
        check_same: js_sys::Function,
    ) -> JsWithoutParents {
        JsWithoutParents {
            ability: dynamic::Dynamic {
                cmd,
                args: (&args).into(),
            },
            config: Config {
                is_nonce_meaningful,
                validate_shape,
                check_same,
            },
        }
    }

    #[wasm_bindgen(getter)]
    pub fn is_nonce_meaningful(&self) -> bool {
        self.config.is_nonce_meaningful
    }

    pub fn check_shape(&self) -> Result<(), JsValue> {
        let this = wasm_bindgen::JsValue::NULL;
        self.config
            .validate_shape
            .call1(&this, &self.ability.args.clone().into())
            .map(|_| ())
    }

    // FIXME throws on Err
    pub fn check_same(&self, proof: &Object) -> Result<(), JsValue> {
        let this = wasm_bindgen::JsValue::NULL;
        self.config
            .check_same
            .call2(&this, &self.ability.args.clone().into(), proof)
            .map(|_| ())
    }
}

// FIXME while this can totally be done by converting to the dynamic carrier type, this seems more straightforward?
impl CheckSame for JsWithoutParents {
    type Error = JsValue;

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        self.check_same(&proof.ability.args.clone().into())
    }
}

impl ToCommand for JsWithoutParents {
    fn to_command(&self) -> String {
        self.ability.cmd.clone()
    }
}

impl From<JsWithoutParents> for Arguments {
    fn from(js: JsWithoutParents) -> Self {
        js.ability.into()
    }
}

impl Checkable for JsWithoutParents {
    type Hierarchy = Parentless<JsWithoutParents>;
}
