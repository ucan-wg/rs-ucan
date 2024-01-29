use super::{
    internal::Checker,
    parents::CheckParents,
    prove::{Outcome, Prove},
    same::CheckSame,
};
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Parentful<T: CheckParents> {
    Any,
    Parents(T::Parents),
    This(T),
}

impl<T: CheckParents> From<Parentful<T>> for Ipld
where
    Ipld: From<T>,
{
    fn from(parentful: Parentful<T>) -> Self {
        parentful.into()
    }
}

impl<T: TryFrom<Ipld> + DeserializeOwned + CheckParents> TryFrom<Ipld> for Parentful<T>
where
    <T as CheckParents>::Parents: DeserializeOwned,
{
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl<T: CheckParents> Checker for Parentful<T> {}

impl<T: CheckParents> Prove<Parentful<T>> for T
where
    T::Parents: CheckSame,
{
    type ArgumentError = T::Error;
    type ProofChainError = T::ParentError;

    fn check<'a>(&'a self, proof: &'a Parentful<T>) -> Outcome<T::Error, T::ParentError> {
        match proof {
            Parentful::Any => Outcome::ProvenByAny,
            Parentful::Parents(parents) => match self.check_parents(parents) {
                Ok(()) => Outcome::Proven,
                Err(e) => Outcome::InvalidProofChain(e),
            },
            Parentful::This(that) => match self.check_same(&that) {
                Ok(()) => Outcome::Proven,
                Err(e) => Outcome::ArgumentEscelation(e),
            },
        }
    }
}
