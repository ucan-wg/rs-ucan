//! High-level proof chain checking.

use super::internal::Checker;

/// An internal trait that checks based on the other traits for an ability type.
pub trait Prove<T: Checker> {
    /// The error if the argument is invalid.
    type ArgumentError;

    /// The error if the proof chain is invalid.
    type ProofChainError;

    /// The error if the parents are invalid.
    type ParentsError;

    fn check(
        &self,
        proof: &T,
    ) -> Outcome<Self::ArgumentError, Self::ProofChainError, Self::ParentsError>;
}

// FIXME that's a lot of error type params
/// The outcome of a proof check.
pub enum Outcome<ArgErr, ChainErr, ParentErr> {
    /// Success
    Proven,

    /// Special case for success by checking against `*`.
    ProvenByAny,

    /// An error in the command chain.
    CommandEscelation,

    /// An error in the argument chain.
    ArgumentEscelation(ArgErr),

    /// An error in the proof chain.
    InvalidProofChain(ChainErr),

    /// An error in the parents.
    InvalidParents(ParentErr),
}
