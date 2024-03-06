//! [UCAN Promise](https://github.com/ucan-wg/promise)s: selectors, wrappers, and traits.

mod any;
mod err;
mod ok;
mod pending;
mod resolvable;

pub mod store;
// FIXME pub mod js;

pub use any::Any;
pub use err::PromiseErr;
pub use ok::PromiseOk;
pub use pending::Pending;
pub use resolvable::*;
pub use store::Store;

use enum_as_inner::EnumAsInner;
use libipld_core::cid::Cid;
use serde::{Deserialize, Serialize};

/// Top-level union of all UCAN Promise options
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, EnumAsInner)]
pub enum Promise<T, E> {
    /// The `ucan/await/ok` promise
    Ok(T),

    /// The `ucan/await/err` promise
    Err(E),

    /// The `ucan/await/ok` promise
    PendingOk(Cid),

    /// The `ucan/await/err` promise
    PendingErr(Cid),

    /// The `ucan/await/*` promise
    PendingAny(Cid),

    /// The `ucan/await` promise
    PendingTagged(Cid),
}
