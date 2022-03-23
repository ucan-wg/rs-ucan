#[macro_use]
extern crate log;

#[cfg(all(target_arch = "wasm32", feature = "web"))]
pub mod web_crypto;

pub mod ed25519;
pub mod rsa;
