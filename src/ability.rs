pub mod traits;
// pub mod wasm;

// FIXME feature flag each?
pub mod crud;
pub mod msg;

// TODO move to crate::wasm?
#[cfg(feature = "wasm")]
pub mod dynamic;

// FIXME macro to derive promise versions & delagted builder versions
// ... also maybe Ipld
