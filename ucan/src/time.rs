#[cfg(target_arch = "wasm32")]
use instant::SystemTime;
#[cfg(not(target_arch = "wasm32"))]
use std::time::SystemTime;

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
