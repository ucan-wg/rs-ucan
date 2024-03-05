//! Abilities describe the semantics of what a UCAN is allowed to do.

pub mod pipe;
pub mod ucan;

#[cfg(feature = "ability-crud")]
pub mod crud;

#[cfg(feature = "ability-msg")]
pub mod msg;

#[cfg(feature = "ability-wasm")]
pub mod wasm;

#[cfg(feature = "ability-preset")]
pub mod preset;

pub mod arguments;
pub mod command;
pub mod parse;

#[cfg(target_arch = "wasm32")]
pub mod js;

pub mod dynamic;
