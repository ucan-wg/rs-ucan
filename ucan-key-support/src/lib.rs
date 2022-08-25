#[macro_use]
extern crate log;

#[cfg(all(target_arch = "wasm32", feature = "web"))]
pub mod web_crypto;

#[cfg(all(target_arch = "wasm32", feature = "web"))]
#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

pub mod ed25519;
pub mod rsa;
