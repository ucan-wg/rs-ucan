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

pub enum ParentfulError<ArgErr, PrfErr, ParErr> {
    CommandEscelation,
    ArgumentEscelation(ArgErr),
    InvalidProofChain(PrfErr),
    InvalidParents(ParErr), // FIXME seems kinda broken -- better naming at least
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

impl<T: CheckParents> CheckSame for Parentful<T>
where
    T::Parents: CheckSame,
{
    type Error = ParentfulError<T::Error, T::ParentError, <T::Parents as CheckSame>::Error>; // FIXME

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        match proof {
            Parentful::Any => Ok(()),
            Parentful::Parents(their_parents) => match self {
                Parentful::Any => Err(ParentfulError::CommandEscelation),
                Parentful::Parents(parents) => parents
                    .check_same(their_parents)
                    .map_err(ParentfulError::InvalidParents),
                Parentful::This(this) => this
                    .check_parents(their_parents)
                    .map_err(ParentfulError::InvalidProofChain),
            },
            Parentful::This(that) => match self {
                Parentful::Any => Err(ParentfulError::CommandEscelation),
                Parentful::Parents(_) => Err(ParentfulError::CommandEscelation),
                Parentful::This(this) => this
                    .check_same(that)
                    .map_err(ParentfulError::ArgumentEscelation),
            },
        }
    }
}

impl<T: CheckParents> CheckParents for Parentful<T>
where
    T::Parents: CheckSame,
{
    type Parents = Parentful<T>;
    type ParentError = ParentfulError<T::Error, T::ParentError, <T::Parents as CheckSame>::Error>;

    fn check_parents(&self, proof: &Parentful<T>) -> Result<(), Self::ParentError> {
        match proof {
            Parentful::Any => Ok(()),
            Parentful::Parents(their_parents) => match self {
                Parentful::Any => Err(ParentfulError::CommandEscelation),
                Parentful::Parents(parents) => parents
                    .check_same(their_parents)
                    .map_err(ParentfulError::InvalidParents),
                Parentful::This(this) => this
                    .check_parents(their_parents)
                    .map_err(ParentfulError::InvalidProofChain),
            },
            Parentful::This(that) => match self {
                Parentful::Any => Err(ParentfulError::CommandEscelation),
                Parentful::Parents(_) => Err(ParentfulError::CommandEscelation),
                Parentful::This(this) => this
                    .check_same(that)
                    .map_err(ParentfulError::ArgumentEscelation),
            },
        }
    }
}

impl<T: CheckParents> Checker for Parentful<T> {}

impl<T: CheckParents> Prove<Parentful<T>> for Parentful<T>
where
    T::Parents: CheckSame,
{
    type ArgumentError = T::Error;
    type ProofChainError = T::ParentError;
    type ParentsError = <T::Parents as CheckSame>::Error; // FIXME better name

    fn check(&self, proof: &Parentful<T>) -> Outcome<T::Error, T::ParentError, Self::ParentsError> {
        match proof {
            Parentful::Any => Outcome::ProvenByAny,
            Parentful::Parents(their_parents) => match self {
                Parentful::Any => Outcome::CommandEscelation,
                Parentful::Parents(parents) => match parents.check_same(their_parents) {
                    Ok(()) => Outcome::Proven,
                    Err(e) => Outcome::InvalidParents(e),
                },
                Parentful::This(this) => match this.check_parents(their_parents) {
                    Ok(()) => Outcome::Proven,
                    Err(e) => Outcome::InvalidProofChain(e),
                },
            },
            Parentful::This(that) => match self {
                Parentful::Any => Outcome::CommandEscelation,
                Parentful::Parents(_) => Outcome::CommandEscelation,
                Parentful::This(this) => match this.check_same(that) {
                    Ok(()) => Outcome::Proven,
                    Err(e) => Outcome::ArgumentEscelation(e),
                },
            },
        }
    }
}
