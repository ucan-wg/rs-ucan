// FIXME feature flag each?
pub mod crud;
pub mod msg;
pub mod ucan;
pub mod wasm;

pub mod arguments;
pub mod command;

// TODO move to crate::wasm?
// #[cfg(feature = "wasm")]
pub mod dynamic;

// FIXME macro to derive promise versions & delagted builder versions
// ... also maybe Ipld
