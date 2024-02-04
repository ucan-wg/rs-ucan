use crate::{
    ability::{arguments::Arguments, command::ToCommand, dynamic},
    ipld,
    proof::{checkable::Checkable, parentful::Parentful, parents::CheckParents, same::CheckSame},
};
use js_sys::{Function, JsString, Map, Object, Reflect};
use libipld_core::ipld::Ipld;
use std::collections::BTreeMap;
use wasm_bindgen::{prelude::*, JsValue};

// NOTE NOTE NOTE: the strategy is: "you (JS) hand us the cfg" AKA strategy,
// and we (Rust) wire it up and run it for you
// NOTE becuase of the above, no need to export JsWithParents to JS
// FIXME rename
type JsWithParents = dynamic::Configured<Config>;

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
    pub check_parents: BTreeMap<String, Box<Function>>,
}

#[wasm_bindgen(typescript_custom_section)]
const CONSTRUCTOR_WITH_MAP: &str = r#"
interface ConfigArgs {
    command: string,
    is_nonce_meaningful: boolean,
    validate_shape: Function,
    check_same: Function,
    check_parents: Map<string, Function>
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "ConfigArgs")]
    pub type ConfigArgs;

    pub fn command(this: &ConfigArgs) -> String;

    pub fn is_nonce_meaningful(this: &ConfigArgs) -> bool;

    pub fn validate_shape(this: &ConfigArgs) -> Function;

    pub fn check_same(this: &ConfigArgs) -> Function;

    pub fn check_parents(this: &ConfigArgs) -> Map;
}

#[wasm_bindgen]
impl Config {
    // FIXME object args as an option
    #[wasm_bindgen(constructor, typescript_type = "ConfigArgs")]
    pub fn new(
        js: ConfigArgs,
        //  command: String,
        //  is_nonce_meaningful: bool,
        //  validate_shape: Function,
        //  check_same: Function,
        //  check_parents: Map, // FIXME swap for an object?
    ) -> Result<Config, JsValue> {
        Ok(Config {
            command: command(&js),
            is_nonce_meaningful: is_nonce_meaningful(&js),
            validate_shape: validate_shape(&js),
            check_same: check_same(&js),
            check_parents: {
                let mut btree = BTreeMap::new();
                let mut acc = Ok(());
                //                    Correct order
                check_parents(&js).for_each(&mut |value, key| {
                    if let Ok(_) = &acc {
                        match key.as_string() {
                            None => acc = Err(JsString::from("Key is not a string")), // FIXME better err
                            Some(str_key) => match value.dyn_ref::<Function>() {
                                None => acc = Err("Value is not a function".into()),
                                Some(f) => {
                                    btree.insert(str_key, Box::new(f.clone()));
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

impl From<JsWithParents> for dynamic::Dynamic {
    fn from(js: JsWithParents) -> Self {
        dynamic::Dynamic {
            cmd: js.config.command,
            args: js.arguments,
        }
    }
}

// FIXME while this can totally be done by converting to the dynamic carrier type, this seems more straightforward?
impl CheckSame for JsWithParents {
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

impl CheckParents for JsWithParents {
    type Parents = dynamic::Dynamic; // FIXME actually no? What if we want to plug in random stuff?
    type ParentError = JsValue;

    fn check_parents(&self, parent: &dynamic::Dynamic) -> Result<(), Self::Error> {
        if let Some(handler) = self.config.check_parents.get(&parent.cmd) {
            let this = wasm_bindgen::JsValue::NULL;
            handler
                .call2(
                    &this,
                    &self.arguments.clone().into(),
                    &parent.args.clone().into(),
                )
                .map(|_| ())
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

impl Checkable for JsWithParents {
    type Hierarchy = Parentful<JsWithParents>;
}
