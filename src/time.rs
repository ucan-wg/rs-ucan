//! Time utilities

use web_time::SystemTime;

/// Get the current time in seconds since UNIX_EPOCH
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs()
}
