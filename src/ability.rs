//! Abilities describe the semantics of what a UCAN is allowed to do.
//!
//! # Top Level Structure
//!
//! They always follow the same format at the top level:
//!
//! | Field  | Name                        | Description                      |
//! |--------|-----------------------------|----------------------------------|
//! | `cmd`  | [Command](command::Command) | Roughly a function name. Determines the shape of the `args`. |
//! | `args` | [Arguments](arguments)      | Roughly the function's arguments |
//!
//! # Proof Hierarchy
//!
//! Any UCAN can be proven by the `*` ability. This has been special-cased
//! into the library, and you don't have to worry about it directly when
//! implementing a new ability.
//!
//! Most abilities have no additional parents. If they do, they follow a
//! strict hierararchy. The [CRUD hierarchy](crate::abilities::crud::Any)
//! is a good example.
//!
//! Not all abilities in the hierarchy are invocable: some abstract over
//! multiple `cmd`s (such as [`crud/*`](crate::abilities::crud::Any) for
//! all CRUD actions). This allows for flexibility in adding more abilities
//! under the same hierarchy in the future without having to reissue all of
//! your certificates.
//!
//! # Lifecycle
//!
//! All abilities start as a delegation, which can omit fields (but must
//! stay the same or add more at each delegatoion). When they are invoked,
//! all field much be present. The only exception is promises, where a
//! field may include a promise pointing at another invocation. Once fully
//! resolved ("ready"), they must be validatable against the delegation chain.

// FIXME feature flag each?
// FIXME ability implementers guide (e.g. serde deny fields)
//
pub mod crud;
pub mod msg;
pub mod ucan;
pub mod wasm;

pub mod arguments;
pub mod command;

#[cfg(target_arch = "wasm32")]
pub mod js;

// // TODO move to crate::wasm? or hide behind "dynamic" feature flag?
pub mod dynamic;

// FIXME macro to derive promise versions & delagted builder versions
// ... also maybe Ipld
