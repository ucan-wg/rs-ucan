//! Configure & attach an ambient environment to a value.
//!
//! See the [`Reader`] struct for more information.

mod generic;
mod promised;

pub use generic::Reader;
pub use promised::Promised;
