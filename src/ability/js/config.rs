//! JavaScript interface for abilities that *do* require a parent hierarchy

use crate::{
    ability::{arguments, command::ToCommand, dynamic},
    reader::Reader,
};
use js_sys::{Function, JsString, Map};
use libipld_core::ipld::Ipld;
use std::collections::BTreeMap;
use wasm_bindgen::{prelude::*, JsValue};

// FIXME rename
type WithParents = Reader<ParentfulConfig, arguments::Named<Ipld>>;

// FIXME just make into a general config?

/// The configuration object that expresses an ability (with parents) from JS
#[derive(Debug, Clone, PartialEq, Default)]
#[wasm_bindgen(getter_with_clone)]
pub struct ParentfulConfig {
    pub command: String,

    #[wasm_bindgen(js_name = isNonceMeaningful)]
    pub is_nonce_meaningful: bool,

    #[wasm_bindgen(js_name = validateShape)]
    pub validate_shape: Function,
}

// NOTE if changed, please update this in the docs for `ParentfulArgs` below
#[wasm_bindgen(typescript_custom_section)]
const PARENTFUL_ARGS: &str = r#"
interface ParentfulArgs {
    command: string,
    isNonceMeaningful: boolean,
    validateShape: Function,
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
    ///    }
    ///  }
    /// );
    /// ```
    #[wasm_bindgen(constructor)]
    pub fn new(js_obj: ParentfulArgs) -> Result<ParentfulConfig, JsValue> {
        Ok(ParentfulConfig {
            command: command(&js_obj),
            is_nonce_meaningful: is_nonce_meaningful(&js_obj),
            validate_shape: validate_shape(&js_obj),
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

impl ToCommand for ParentfulConfig {
    fn to_command(&self) -> String {
        self.command.clone()
    }
}

impl Checkable for WithParents {
    type Hierarchy = Parentful<WithParents>;
}
