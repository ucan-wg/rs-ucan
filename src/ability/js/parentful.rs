use crate::{
    ability::{arguments::Arguments, command::ToCommand, dynamic},
    ipld,
    proof::{checkable::Checkable, parentful::Parentful, parents::CheckParents, same::CheckSame},
};
use js_sys::{Function, Map, Object, Reflect};
use libipld_core::ipld::Ipld;
use std::collections::BTreeMap;
use wasm_bindgen::{prelude::*, JsValue};

// FIXME rename to module
#[derive(Debug, Clone, PartialEq)]
#[wasm_bindgen]
pub struct JsWithParents {
    // FIXME just inline and use from
    #[wasm_bindgen(skip)]
    pub ability: dynamic::Dynamic,

    #[wasm_bindgen(skip)]
    pub config: Config,

    #[wasm_bindgen(skip)]
    pub check_parents: Function,
}

// FIXME just inline
#[derive(Debug, Clone, PartialEq)]
pub struct Config {
    pub is_nonce_meaningful: bool,
    pub validate_shape: Function,
    pub check_same: Function,
}

#[wasm_bindgen]
impl JsWithParents {
    // FIXME consider using an object with named fields
    // FIXME needs borrows?
    #[wasm_bindgen(constructor)]
    pub fn new(
        cmd: String,
        args: Object,
        is_nonce_meaningful: bool,
        validate_shape: js_sys::Function, // FIXME Need to actuallyrun this on create
        check_same: js_sys::Function,
        check_parents: js_sys::Function, // FIXME what is an object? i.e. {cmd: handler}?
    ) -> JsWithParents {
        JsWithParents {
            ability: dynamic::Dynamic {
                cmd,
                args: (&args).into(),
            },
            config: Config {
                is_nonce_meaningful,
                validate_shape,
                check_same,
            },
            check_parents,
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

    // FIXME work with native Rust abilities, too
    pub fn check_parents(&self, parent: &Object) -> Result<(), JsValue> {
        let this = wasm_bindgen::JsValue::NULL;
        self.check_parents
            .call2(&this, &self.ability.args.clone().into(), parent)
            .map(|_| ())
    }
}

// FIXME while this can totally be done by converting to the dynamic carrier type, this seems more straightforward?
impl CheckSame for JsWithParents {
    type Error = JsValue;

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        self.check_same(&proof.ability.args.clone().into())
    }
}

impl CheckParents for JsWithParents {
    type Parents = dynamic::Dynamic;
    type ParentError = JsValue;

    fn check_parents(&self, parent: &dynamic::Dynamic) -> Result<(), Self::Error> {
        let obj = Object::new();
        Reflect::set(&obj, &"cmd".into(), &parent.cmd.clone().into())?;
        Reflect::set(&obj, &"args".into(), &parent.args.clone().into())?;

        self.check_parents(&obj)
    }
}

impl ToCommand for JsWithParents {
    fn to_command(&self) -> String {
        self.ability.cmd.clone()
    }
}

impl From<JsWithParents> for Arguments {
    fn from(js: JsWithParents) -> Self {
        js.ability.into()
    }
}

impl Checkable for JsWithParents {
    type Hierarchy = Parentful<JsWithParents>;
}
