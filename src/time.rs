//! Time utilities.
//!
//! The [`Timestamp`] struct is the main type for representing time in a UCAN token.

mod error;
mod js;
mod timestamp;

pub use error::{OutOfRangeError, TimeBoundError};
pub use js::JsTime;
pub use timestamp::Timestamp;
