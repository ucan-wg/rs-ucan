//! [UCAN Promise](https://github.com/ucan-wg/promise)

// FIXME put entire module behind feature flag

mod any;
mod err;
mod ok;
mod resolves;

pub mod js;

pub use any::PromiseAny;
pub use err::PromiseErr;
pub use ok::PromiseOk;
pub use resolves::Resolves;

use serde::{Deserialize, Serialize};

/// Top-level union of all UCAN Promise options
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Promise<T, E> {
    /// The `await/ok` promise
    Ok(PromiseOk<T>),

    /// The `await/err` promise
    Err(PromiseErr<E>),

    /// The `await/*` promise
    Any(PromiseAny<T, E>),
}
