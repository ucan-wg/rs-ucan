#[cfg(not(target_arch = "wasm32"))]
pub fn now() -> u64 {
    use std::time::SystemTime;
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

#[cfg(target_arch = "wasm32")]
pub fn now() -> u64 {
    use js_sys::Date;
    (Date::now() / 1000.0) as u64
}
