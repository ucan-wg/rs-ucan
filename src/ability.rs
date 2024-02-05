// FIXME feature flag each?
pub mod crud;
pub mod msg;
pub mod ucan;
pub mod wasm;

pub mod arguments;
pub mod command;

#[cfg(target_arch = "wasm32")]
pub mod js;

// // TODO move to crate::wasm? or hide behind "dynamic" feature flag?
#[cfg(target_arch = "wasm32")]
pub mod dynamic;

// FIXME macro to derive promise versions & delagted builder versions
// ... also maybe Ipld
