use crate::proof::checkable::Checkable;

/// A trait for types that can be delegated.
///
/// Since [`Delegation`]s may omit fields (until [`Invocation`]),
/// this trait helps associate the delegatable variant to the invocable one.
///
/// [`Delegation`]: crate::delegation::Delegation
/// [`Invocation`]: crate::invocation::Invocation
pub trait Delegable: Sized {
    /// A delegation with some arguments filled.
    type Builder: TryInto<Self> + From<Self> + Checkable;
}
