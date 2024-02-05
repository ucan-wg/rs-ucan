use crate::{
    ability::{arguments::Arguments, command::ToCommand, dynamic},
    proof::{parentless::NoParents, same::CheckSame},
    reader::Reader,
};
use js_sys::Function;
use wasm_bindgen::prelude::*;

// NOTE NOTE NOTE: the strategy is: "you (JS) hand us the cfg" AKA strategy,
// and we (Rust) wire it up and run it for you
// NOTE becuase of the above, no need to export JsWithParents to JS
// FIXME rename
type WithoutParents = Reader<ParentlessConfig, Arguments>;

// FIXME rename ability? abilityconfig? leave as is?
// #[wasm_bindgen(getter_with_clone)]
#[derive(Debug, Clone, PartialEq)]
#[wasm_bindgen]
pub struct ParentlessConfig {
    command: String,
    is_nonce_meaningful: bool,
    validate_shape: Function,
    check_same: Function,
}

// FIXME represent promises (for Promised) and options (for builder)

#[wasm_bindgen(typescript_custom_section)]
const PARENTLESS_ARGS: &str = r#"
interface ParentlessArgs {
    command: string,
    is_nonce_meaningful: boolean,
    validate_shape: Function,
    check_same: Function,
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "ParentlessArgs")]
    pub type ParentlessArgs;

    pub fn command(this: &ParentlessArgs) -> String;
    pub fn is_nonce_meaningful(this: &ParentlessArgs) -> bool;
    pub fn validate_shape(this: &ParentlessArgs) -> Function;
    pub fn check_same(this: &ParentlessArgs) -> Function;
}

#[wasm_bindgen]
impl ParentlessConfig {
    // FIXME object args as an option
    #[wasm_bindgen(constructor)]
    pub fn new(js_obj: ParentlessArgs) -> ParentlessConfig {
        ParentlessConfig {
            command: command(&js_obj),
            is_nonce_meaningful: is_nonce_meaningful(&js_obj),
            validate_shape: validate_shape(&js_obj),
            check_same: check_same(&js_obj),
        }
    }
}

impl From<WithoutParents> for dynamic::Dynamic {
    fn from(js: WithoutParents) -> Self {
        dynamic::Dynamic {
            cmd: js.env.command,
            args: js.val,
        }
    }
}

// FIXME while this can totally be done by converting to the dynamic carrier type, this seems more straightforward?
impl CheckSame for WithoutParents {
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

impl ToCommand for ParentlessConfig {
    fn to_command(&self) -> String {
        self.command.clone()
    }
}

impl NoParents for ParentlessConfig {}
