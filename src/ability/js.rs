use super::{arguments::Arguments, dynamic};
use crate::{ipld, proof::same::CheckSame};
use js_sys::Object;
use libipld_core::ipld::Ipld;
use std::collections::BTreeMap;
use wasm_bindgen::prelude::*;

// FIXME now we can just use dynamic again (yay)
#[wasm_bindgen]
pub struct Ability {
    cmd: String, // FIXME don't need this field because it's on the validator?
    // FIXME JsCast for Args or WrappedIpld, esp for Cids
    args: Arguments, // FIXME args
                     // pub args: wasm_bindgen::JsValue, // js_sys::Object, // BTreeMap<String, JsValue>, // FIXME args
}

impl From<Ability> for js_sys::Map {
    fn from(ability: Ability) -> Self {
        let args = js_sys::Map::new();
        for (k, v) in ability.args.0 {
            args.set(&k.into(), &ipld::Newtype(v).into());
        }

        let map = js_sys::Map::new();
        map.set(&"args".into(), &js_sys::Object::from(args).into());
        map.set(&"cmd".into(), &ability.cmd.into());
        map.into()
    }
}

impl TryFrom<js_sys::Map> for Ability {
    type Error = JsValue;

    fn try_from(map: js_sys::Map) -> Result<Self, Self::Error> {
        if let (Some(cmd), js_args) = (
            map.get(&("cmd".into())).as_string(),
            &map.get(&("args".into())),
        ) {
            let obj_args = js_sys::Object::try_from(js_args).ok_or(wasm_bindgen::JsValue::NULL)?;
            let keys = Object::keys(obj_args);
            let values = Object::values(obj_args);

            let mut btree = BTreeMap::new();
            for (k, v) in keys.iter().zip(values) {
                if let Some(k) = k.as_string() {
                    btree.insert(k, ipld::Newtype::try_from(v).expect("FIXME").0);
                } else {
                    return Err(k);
                }
            }

            Ok(Ability {
                cmd,
                args: Arguments(btree), // FIXME kill clone
            })
        } else {
            Err(JsValue::NULL) // FIXME
        }
    }
}

pub type Abc = dynamic::ValidateWithoutParents<js_sys::Function, js_sys::Function>;
pub type Xyz = dynamic::ValidateWithParents<js_sys::Function, js_sys::Function, js_sys::Function>;

// #[wasm_bindgen]
// #[derive(Debug, Clone, PartialEq)]
// pub struct ValidateWithoutParents {
//     ability: dynamic::Dynamic,
//
//     #[wasm_bindgen(skip)]
//     config: u32, // dynamic::Config0<js_sys::Function, js_sys::Function>,
// }
//
// // pub struct ValidateWithParents {
// //     ability: Dynamic,
// //     config: Config1<js_sys::Function, js_sys::Function, js_sys::Function>,
// // }
//
// #[wasm_bindgen]
// impl ValidateWithoutParents {
//     pub fn foo(x: u32) -> ValidateWithoutParents {
//         todo!()
//     }
// }

// #[wasm_bindgen]
// #[derive(Debug, Clone, PartialEq)]
// pub struct Validator {
//     #[wasm_bindgen(skip)]
//     pub cmd: String,
//
//     #[wasm_bindgen(readonly)]
//     pub is_nonce_meaningful: bool,
//
//     #[wasm_bindgen(skip)]
//     pub validate_shape: js_sys::Function,
//
//     #[wasm_bindgen(skip)]
//     pub check_same: js_sys::Function,
//
//     #[wasm_bindgen(skip)]
//     pub check_parent: Option<js_sys::Function>, // FIXME explore concrete types + an enum
// }

// Helper
pub fn invoke(f: &js_sys::Function, args: Vec<JsValue>) -> Result<JsValue, JsValue> {
    // FIXME annoying number of steps... so I guess that's why they have the numbered versions...
    // but those end at 3 :/
    // Hmm I guess this is reasonable, since it needs to copy the `Vec` to the JsArray
    let arr = js_sys::Array::new_with_length(args.len() as u32);
    for (i, arg) in args.iter().enumerate() {
        arr.set(i as u32, arg.clone());
    }

    f.apply(&wasm_bindgen::JsValue::NULL, &arr)
}

// // NOTE more like a config object
// #[wasm_bindgen]
// impl Validator {
//     // FIXME wrap in func that checks the jsval or better: converts form Ipld
//     // FIXME notes about checking shape on the way in
//     #[wasm_bindgen(constructor)]
//     pub fn new(
//         cmd: String,
//         is_nonce_meaningful: bool,
//         validate_shape: js_sys::Function,
//         check_same: js_sys::Function,
//         check_parent: Option<js_sys::Function>,
//     ) -> Validator {
//         // FIXME chec that JsErr doesn't auto-throw
//         Validator {
//             cmd,
//             is_nonce_meaningful,
//             validate_shape,
//             check_same,
//             check_parent,
//         }
//     }
//
//     pub fn command(&self) -> String {
//         self.cmd.clone()
//     }
//
//     // e.g. reject extra fields
//     pub fn validate_shape(&self, args: &wasm_bindgen::JsValue) -> Result<(), JsValue> {
//         let this = wasm_bindgen::JsValue::NULL;
//         self.validate_shape.call1(&this, args)?;
//         Ok(())
//     }
//
//     // FIXME only dynamic?
//     pub fn check_same(
//         &self,
//         target: &js_sys::Object,
//         proof: &js_sys::Object,
//     ) -> Result<(), JsValue> {
//         let this = wasm_bindgen::JsValue::NULL;
//         self.check_same.call2(&this, target, proof)?;
//         Ok(())
//     }
//
//     pub fn check_parents(
//         &self,
//         target: &js_sys::Object, // FIXME better type, esp for TS?
//         proof: &js_sys::Object,
//     ) -> Result<(), JsValue> {
//         let this = wasm_bindgen::JsValue::NULL;
//         if let Some(checker) = &self.check_parent {
//             checker.call2(&this, target, proof)?;
//             return Ok(());
//         }
//
//         Err(this)
//     }
// }
//
// pub struct Quux<T> {
//     quux: T,
// }
//
// type Bez = Quux<u32>;
//
// #[wasm_bindgen]
// impl Bez {}
//
// pub struct Baz<T> {
//     pub ability: Ability,
//     pub validator: T,
// }
//
// pub struct Foo {
//     pub ability: Ability,
//     pub validator: Validator,
// }
//
// impl From<Foo> for Arguments {
//     fn from(foo: Foo) -> Self {
//         todo!() // FIXME
//     }
// }
//
// use crate::delegation::Delegatable;
//
// impl Delegatable for Foo {
//     type Builder = Foo;
// }
//
// impl CheckSame for Foo {
//     type Error = JsValue;
//
//     fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
//         let this_it = self.ability.args.iter().map(|(k, v)| (JsValue::from(k), v));
//
//         let mut this_args = js_sys::Map::new();
//         for (k, v) in this_it {
//             this_args.set(&k, &ipld::Newtype(v.clone()).into());
//         }
//
//         let proof_it = proof
//             .ability
//             .args
//             .iter()
//             .map(|(k, v)| (JsValue::from(k), v));
//
//         let mut proof_args = js_sys::Map::new();
//         for (k, v) in proof_it {
//             proof_args.set(&k, &ipld::Newtype(v.clone()).into());
//         }
//
//         self.validator.check_same(
//             &Object::from_entries(&this_args)?,
//             &Object::from_entries(&proof_args)?,
//         )
//     }
// }
//
// // pub struct Ability {
// //     pub cmd: String,
// //     pub args: BTreeMap<String, Ipld>, // FIXME args
// //     pub val: JsValidator,
// // }
// //
// // #[wasm_bindgen]
// // impl Ability {
// //     #[wasm_bindgen(constructor)]
// //     fn new(
// //         cmd: String,
// //         args: BTreeMap<String, JsValue>,
// //         validator: JsValidator,
// //     ) -> Result<Self, JsError> {
// //         let args = args
// //             .iter()
// //             .map(|(k, v)| (k.clone(), JsValue::from(v.clone())))
// //             .collect();
// //
// //         validator.check_shape(args)?;
// //         Ok(Ability { cmd, args, val })
// //     }
// // }
// //
// // pub struct Pipeline {
// //     pub validators: Vec<JsValidator>,
// // }
