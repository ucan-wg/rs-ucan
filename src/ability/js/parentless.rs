//! JavaScript interface for abilities that *do not* require a parent hierarchy

use crate::{
    ability::{arguments, command::ToCommand, dynamic},
    proof::{parentless::NoParents, same::CheckSame},
    reader::Reader,
};
use js_sys::Function;
use wasm_bindgen::prelude::*;

// FIXME rename
type WithoutParents = Reader<ParentlessConfig, arguments::Named>;

/// The configuration object that expresses an ability (without parents) from JS
#[derive(Debug, Clone, PartialEq)]
#[wasm_bindgen]
pub struct ParentlessConfig {
    command: String,
    is_nonce_meaningful: bool,
    validate_shape: Function,
    check_same: Function,
}

// FIXME represent promises (for Promised) and options (for builder)

// NOTE if changed, please update this in the docs for `ParentlessArgs` below
#[wasm_bindgen(typescript_custom_section)]
pub const PARENTLESS_ARGS: &str = r#"
interface ParentlessArgs {
    command: string,
    isNonceMeaningful: boolean,
    validateShape: Function,
    checkSame: Function,
}
"#;

#[wasm_bindgen]
extern "C" {
    /// Named constructor arguments for `ParentlessConfig`
    ///
    /// This forms the basis for configuring an ability.
    /// These values will be used at runtime to perform
    /// checks on the ability (e.g. during delegation),
    /// for indexing, and storage (among others).
    ///
    /// ```typescript
    /// // TypeScript
    /// interface ParentlessArgs {
    ///   command: string,
    ///   isNonceMeaningful: boolean,
    ///   validateShape: Function,
    ///   checkSame: Function,
    /// }
    /// ```
    #[wasm_bindgen(typescript_type = "ParentlessArgs")]
    pub type ParentlessArgs;

    /// Get the [`Command`][crate::ability::command::Command] string
    #[wasm_bindgen(js_name = command)]
    pub fn command(this: &ParentlessArgs) -> String;

    /// Whether the nonce should factor into a receipt's global index ([`task::Id`])
    #[wasm_bindgen(js_name = isNonceMeaningful)]
    pub fn is_nonce_meaningful(this: &ParentlessArgs) -> bool;

    /// Parser validator
    #[wasm_bindgen(js_name = validateShape)]
    pub fn validate_shape(this: &ParentlessArgs) -> Function;

    /// Validate an instance against a candidate proof of the same shape
    #[wasm_bindgen(js_name = checkSame)]
    pub fn check_same(this: &ParentlessArgs) -> Function;
}

#[wasm_bindgen]
impl ParentlessConfig {
    /// Construct a new `ParentlessConfig` from JavaScript
    ///
    /// # Examples
    ///
    /// ```javascript
    /// // JavaScript
    /// const msgSendConfig = new ParentlessConfig({
    ///    command: "msg/send",
    ///    isNonceMeaningful: true,
    ///    validateShape: (args) => {
    ///      if (args.to && args.message && args.length() === 2) {
    ///        return true;
    ///      }
    ///      return false;
    ///    },
    ///    checkSame: (proof, args) => {
    ///      proof.to === args.to && proof.message === args.message
    ///    },
    ///  }
    /// );
    /// ```
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
                &arguments::Named::from(proof.clone()).into(),
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
