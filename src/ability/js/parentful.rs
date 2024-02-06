//! JavaScript interafce for abilities that *do* require a parent hierarchy

use crate::{
    ability::{arguments, command::ToCommand, dynamic},
    proof::{checkable::Checkable, parentful::Parentful, parents::CheckParents, same::CheckSame},
    reader::Reader,
};
use js_sys::{Function, JsString, Map};
use std::collections::BTreeMap;
use wasm_bindgen::{prelude::*, JsValue};

// FIXME rename
type WithParents = Reader<ParentfulConfig, arguments::Named>;

// Promise = Promise? Ah, nope becuase we need that CID on the promise
// FIXME represent promises (for Promised) and options (for builder)

/// The configuration object that expresses an ability (with parents) from JS
#[derive(Debug, Clone, PartialEq, Default)]
#[wasm_bindgen(getter_with_clone)]
pub struct ParentfulConfig {
    pub command: String,
    pub is_nonce_meaningful: bool,

    pub validate_shape: Function,
    pub check_same: Function,

    #[wasm_bindgen(skip)]
    pub check_parent: BTreeMap<String, Function>,
}

// NOTE if changed, please update this in the docs for `ParentfulArgs` below
#[wasm_bindgen(typescript_custom_section)]
const PARENTFUL_ARGS: &str = r#"
interface ParentfulArgs {
    command: string,
    isNonceMeaningful: boolean,
    validateShape: Function,
    checkSame: Function,
    checkParent: Map<string, Function>
}
"#;

#[wasm_bindgen]
extern "C" {
    /// Named constructor arguments for `ParentfulConfig`
    ///
    /// This forms the basis for configuring an ability.
    /// These values will be used at runtime to perform
    /// checks on the ability (e.g. during delegation),
    /// for indexing, and storage (among others).
    ///
    /// ```typescript
    /// // TypeScript
    /// interface ParentfulArgs {
    ///   command: string,
    ///   isNonceMeaningful: boolean,
    ///   validateShape: Function,
    ///   checkSame: Function,
    ///   checkParent: Map<string, Function>
    /// }
    /// ```
    #[wasm_bindgen(typescript_type = "ParentfulArgs")]
    pub type ParentfulArgs;

    /// Get the [`Command`][crate::ability::command::Command] string
    #[wasm_bindgen(js_name = command)]
    pub fn command(this: &ParentfulArgs) -> String;

    /// Whether the nonce should factor into a receipt's global index ([`task::Id`])
    #[wasm_bindgen(js_name = isNonceMeaningful)]
    pub fn is_nonce_meaningful(this: &ParentfulArgs) -> bool;

    /// Parser validator
    #[wasm_bindgen(js_name = validateShape)]
    pub fn validate_shape(this: &ParentfulArgs) -> Function;

    /// Validate an instance against a candidate proof of the same shape
    #[wasm_bindgen(js_name = checkSame)]
    pub fn check_same(this: &ParentfulArgs) -> Function;

    /// Validate an instance against a candidate proof containing a parent
    #[wasm_bindgen(js_name = checkParent)]
    pub fn check_parent(this: &ParentfulArgs) -> Map;
}

#[wasm_bindgen]
impl ParentfulConfig {
    /// Construct a new `ParentfulConfig` from JavaScript
    ///
    /// # Examples
    ///
    /// ```javascript
    /// // JavaScript
    /// const msgSendConfig = new ParentfulConfig({
    ///    command: "msg/send",
    ///    isNonceMeaningful: true,
    ///    validateShape: (args) => {
    ///      if (args.to && args.message && args.length() === 2) {
    ///        return true;
    ///      }
    ///      return false;
    ///    },
    ///    checkSame: (proof, args) => {
    ///      if (proof.to === args.to && proof.message === args.message) {
    ///        return true;
    ///      }
    ///      return false;
    ///    },
    ///    checkParent: new Map([
    ///      ["msg/*", (proof, args) => {
    ///        proof.to === args.to && proof.message === args.message
    ///      }]
    ///    ])
    ///  }
    /// );
    /// ```
    #[wasm_bindgen(constructor)]
    pub fn new(js_obj: ParentfulArgs) -> Result<ParentfulConfig, JsValue> {
        Ok(ParentfulConfig {
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
                &arguments::Named::from(proof.clone()).into(),
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

impl ToCommand for ParentfulConfig {
    fn to_command(&self) -> String {
        self.command.clone()
    }
}

impl Checkable for WithParents {
    type Hierarchy = Parentful<WithParents>;
}
