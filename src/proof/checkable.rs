//! Define the hierarchy of an ability (or mark as not having one)

use super::{internal::Checker, prove::Prove, same::CheckSame};

// FIXME move to Delegatbel?

/// Plug a type into the delegation checking pipeline
pub trait Checkable: CheckSame {
    /// The type of hierarchy this ability has
    ///
    /// The only options are [`Parentful`][super::parentful::Parentful]
    /// and [`Parentless`][super::parentless::Parentless],
    /// (which are the only instances of the unexported `Checker`)
    type Hierarchy: Checker + CheckSame + Prove<Self::Hierarchy>;
}
