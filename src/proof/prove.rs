use super::internal::Checker;

// FIXME is it worth locking consumers out with that Checker bound?
pub trait Prove<T: Checker> {
    type ArgumentError;
    type ProofChainError;

    fn check<'a>(&'a self, proof: &'a T) -> Outcome<Self::ArgumentError, Self::ProofChainError>;
}

pub enum Outcome<ArgErr, ChainErr> {
    Proven,
    ProvenByAny,
    ArgumentEscelation(ArgErr),
    InvalidProofChain(ChainErr),
}
