//! Utilities for ability arguments

mod named;

pub use named::{Named, NamedError};

use crate::{invocation::promise::Resolves, ipld};

// FIXME move under invoc::promise?
pub type Promised = Resolves<Named<ipld::Promised>>;
