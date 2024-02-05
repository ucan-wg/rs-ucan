use crate::{
    ability::{arguments::Arguments, command::ToCommand, dynamic},
    proof::{checkable::Checkable, parentful::Parentful, parents::CheckParents, same::CheckSame},
    reader::Reader,
};
use js_sys::{Function, JsString, Map};
use std::collections::BTreeMap;
use wasm_bindgen::{prelude::*, JsValue};

// NOTE NOTE NOTE: the strategy is: "you (JS) hand us the cfg" AKA strategy,
// and we (Rust) wire it up and run it for you
// NOTE becuase of the above, no need to export WithParents to JS
// FIXME rename
type WithParents = Reader<Config, Arguments>;

// Promise = Promise? Ah, nope becuase we need that CID on the promise
// FIXME represent promises (for Promised) and options (for builder)

// FIXME rename ability? abilityconfig? leave as is?
#[derive(Debug, Clone, PartialEq, Default)]
#[wasm_bindgen(getter_with_clone)]
pub struct Config {
    pub command: String,
    pub is_nonce_meaningful: bool,

    pub validate_shape: Function,
    pub check_same: Function,

    #[wasm_bindgen(skip)]
    pub check_parent: BTreeMap<String, Function>,
}

#[wasm_bindgen(typescript_custom_section)]
const CONFIG_ARGS: &str = r#"
interface ParentfulArgs {
    command: string,
    is_nonce_meaningful: boolean,
    validate_shape: Function,
    check_same: Function,
    check_parent: Map<string, Function>
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "ParentfulArgs")]
    pub type ParentfulArgs;

    pub fn command(this: &ParentfulArgs) -> String;
    pub fn is_nonce_meaningful(this: &ParentfulArgs) -> bool;
    pub fn validate_shape(this: &ParentfulArgs) -> Function;
    pub fn check_same(this: &ParentfulArgs) -> Function;
    pub fn check_parent(this: &ParentfulArgs) -> Map;
}

#[wasm_bindgen]
impl Config {
    // FIXME object args as an option
    #[wasm_bindgen(constructor)]
    pub fn new(js_obj: ParentfulArgs) -> Result<Config, JsValue> {
        Ok(Config {
            command: command(&js_obj),
            is_nonce_meaningful: is_nonce_meaningful(&js_obj),
            validate_shape: validate_shape(&js_obj),
            check_same: check_same(&js_obj),
            check_parent: {
                let mut btree = BTreeMap::new();
                let mut acc = Ok(());

                check_parent(&js_obj).for_each(&mut |value, key| {
                    // |value, key| is correct ------^^^^^^^^^^^^
                    if let Ok(_) = &acc {
                        match key.as_string() {
                            None => acc = Err(JsString::from("Key is not a string")), // FIXME better err
                            Some(str_key) => match value.dyn_ref::<Function>() {
                                None => acc = Err("Value is not a function".into()),
                                Some(f) => {
                                    btree.insert(str_key, f.clone());
                                    acc = Ok(());
                                }
                            },
                        }
                    }
                });

                acc.map(|_| btree)?
            },
        })
    }
}

impl From<WithParents> for dynamic::Dynamic {
    fn from(js: WithParents) -> Self {
        dynamic::Dynamic {
            cmd: js.env.command,
            args: js.val,
        }
    }
}

// FIXME while this can totally be done by converting to the dynamic carrier type, this seems more straightforward?
impl CheckSame for WithParents {
    type Error = JsValue;

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        let this = wasm_bindgen::JsValue::NULL;
        self.env
            .check_same
            .call2(
                &this,
                &self.val.clone().into(),
                &Arguments::from(proof.clone()).into(),
            )
            .map(|_| ())
    }
}

impl CheckParents for WithParents {
    type Parents = dynamic::Dynamic;
    type ParentError = JsValue;

    fn check_parent(&self, parent: &dynamic::Dynamic) -> Result<(), Self::Error> {
        if let Some(handler) = self.env.check_parent.get(&parent.cmd) {
            let this = wasm_bindgen::JsValue::NULL;
            handler
                .call2(&this, &self.val.clone().into(), &parent.args.clone().into())
                .map(|_| ()) // FIXME
        } else {
            Err(JsValue::from("No handler for parent"))
        }
    }
}

impl ToCommand for Config {
    fn to_command(&self) -> String {
        self.command.clone()
    }
}

impl Checkable for WithParents {
    type Hierarchy = Parentful<WithParents>;
}
