//! Check the parents in an ability hierarchy.

use super::same::CheckSame;

/// Check if the parents of a proof are valid.
///
/// Note that the top ability (`*`) does not need to be handled separately,
/// as the code from [`CheckParents`] will be lifted into
/// [`Parentful`][super::parentful::Parentful], which knows
/// how to check `*`.
pub trait CheckParents: CheckSame {
    /// The parents of the hierarchy.
    ///
    /// Note that `Self` *need not* be included in [`CheckParents::Parents`].
    type Parents;

    /// Error checking against [`CheckParents::Parents`].
    type ParentError;

    // FIXME
    fn check_parent(&self, proof: &Self::Parents) -> Result<(), Self::ParentError>;
}
