//! Time utilities.
//!
//! The [`Timestamp`] struct is the main type for representing time in a UCAN token.

mod error;
mod timestamp;

pub use error::*;
pub use timestamp::Timestamp;
