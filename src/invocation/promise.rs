//! [UCAN Promise](https://github.com/ucan-wg/promise)s: selectors, wrappers, and traits.

// FIXME put entire module behind feature flag

mod any;
mod err;
mod ok;
mod resolvable;
mod resolves;

pub mod store;
// FIXME pub mod js;

pub use any::PromiseAny;
pub use err::PromiseErr;
pub use ok::PromiseOk;
pub use resolvable::Resolvable;
pub use resolves::Resolves;
pub use store::Store;

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

impl<T, E> From<PromiseOk<T>> for Promise<T, E> {
    fn from(p_ok: PromiseOk<T>) -> Self {
        Promise::Ok(p_ok)
    }
}

impl<T, E> From<PromiseErr<E>> for Promise<T, E> {
    fn from(p_err: PromiseErr<E>) -> Self {
        Promise::Err(p_err)
    }
}

impl<T, E> From<PromiseAny<T, E>> for Promise<T, E> {
    fn from(p_any: PromiseAny<T, E>) -> Self {
        Promise::Any(p_any)
    }
}
