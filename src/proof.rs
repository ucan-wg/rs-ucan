//! Proof chains, checking, and utilities.

pub mod checkable;
pub mod error;
pub mod parentful;
pub mod parentless;
pub mod parents;
pub mod prove;
pub mod same;
pub mod util;

// NOTE must remain *un*exported!
pub(super) mod internal;
