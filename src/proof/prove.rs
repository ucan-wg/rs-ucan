//! High-level proof chain checking.

use super::internal::Checker;

// FIXME move to internal?

/// An internal trait that checks based on the other traits for an ability type.
pub trait Prove: Checker {
    type Error;

    // FIXME make the same as the trait name (prove)
    fn check(&self, proof: &Self) -> Result<Success, Self::Error>;
}

#[derive(Debug, Clone, PartialEq)]
pub enum Success {
    /// Success
    Proven,

    /// Special case for success by checking against `*`.
    ProvenByAny,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Failure<ArgErr, ChainErr, ParentErr> {
    /// An error in the command chain.
    CommandEscelation,

    /// An error in the argument chain.
    ArgumentEscelation(ArgErr),

    /// An error in the proof chain.
    InvalidProofChain(ChainErr),

    /// An error in the parents.
    InvalidParents(ParentErr),
}
