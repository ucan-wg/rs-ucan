//! Configure & attach an ambient environment to a value.
//!
//! See the [`Reader`] struct for more information.

mod builder;
mod generic;
mod promised;

pub use builder::Builder;
pub use generic::Reader;
pub use promised::Promised;
