// FIXME feature flag each?
pub mod crud;
pub mod msg;
pub mod ucan;
pub mod wasm;

pub mod arguments;
pub mod command;

// // TODO move to crate::wasm? or hide behind feature flag?
#[cfg(target_arch = "wasm32")]
pub mod dynamic;

// FIXME macro to derive promise versions & delagted builder versions
// ... also maybe Ipld
