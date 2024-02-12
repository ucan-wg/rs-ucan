//! High-level proof chain checking.

use super::internal::Checker;

/// An internal trait that checks based on the other traits for an ability type.
pub(crate) trait Prove: Checker {
    type Error;

    fn check(&self, proof: &Self) -> Result<Success, Self::Error>;
}

pub enum Success {
    /// Success
    Proven,

    /// Special case for success by checking against `*`.
    ProvenByAny,
}

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
