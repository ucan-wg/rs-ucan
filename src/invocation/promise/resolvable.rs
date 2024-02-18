use crate::{ability::arguments, delegation::Delegable};
use libipld_core::ipld::Ipld;

// FIXME rename "Unresolved"
// FIXME better name

/// A trait for [`Delegable`]s that can be deferred (by promises).
///
/// FIXME exmaples
pub trait Resolvable: Delegable {
    /// The promise type that resolves to `Self`.
    ///
    /// Note that this may be a more complex type than the promise selector
    /// variants. One example is [letting any leaf][PromiseIpld] of an [`Ipld`] graph
    /// be a promise.
    ///
    /// [PromiseIpld]: crate::ipld::Promised
    type Promised: Into<Self::Builder> + Into<arguments::Named<Ipld>>;

    /// Attempt to resolve the [`Self::Promised`].
    fn try_resolve(promised: Self::Promised) -> Result<Self, Self::Promised>;
}
