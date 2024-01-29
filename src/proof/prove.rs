use super::internal::Checker;

// FIXME is it worth locking consumers out with that Checker bound?
pub trait Prove<T: Checker> {
    type ArgumentError;
    type ProofChainError;
    type ParentsError;

    fn check(
        &self,
        proof: &T,
    ) -> Outcome<Self::ArgumentError, Self::ProofChainError, Self::ParentsError>;
}

// FIXME that's a lot of error type params
pub enum Outcome<ArgErr, ChainErr, ParentErr> {
    Proven,
    ProvenByAny,
    ArgumentEscelation(ArgErr),
    InvalidProofChain(ChainErr),
    InvalidParents(ParentErr),
    CommandEscelation,
}
