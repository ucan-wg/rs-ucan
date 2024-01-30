use super::{
    checkable::Checkable,
    internal::Checker,
    prove::{Outcome, Prove},
    same::CheckSame,
};
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::convert::Infallible;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Parentless<T> {
    Any,
    This(T),
}

// FIXME generally useful (e.g. checkiung `_/*`); move to its own module and rename
#[derive(Debug, Clone, PartialEq)]
pub enum ParentlessError<T: CheckSame> {
    CommandEscelation,
    ArgumentEscelation(T::Error),
}

// FIXME better name
pub trait NoParents {}

impl<T: NoParents + CheckSame> Checkable for T {
    type Hierarchy = Parentless<Self>;
}

impl<T> From<Parentless<T>> for Ipld
where
    Ipld: From<T>,
{
    fn from(parentless: Parentless<T>) -> Self {
        parentless.into()
    }
}

impl<T: TryFrom<Ipld> + DeserializeOwned> TryFrom<Ipld> for Parentless<T> {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl<T: CheckSame> CheckSame for Parentless<T> {
    type Error = ParentlessError<T>;

    fn check_same(&self, other: &Self) -> Result<(), Self::Error> {
        match other {
            Parentless::Any => Ok(()),
            Parentless::This(that) => match self {
                Parentless::Any => Err(ParentlessError::CommandEscelation),
                Parentless::This(this) => this
                    .check_same(that)
                    .map_err(ParentlessError::ArgumentEscelation),
            },
        }
    }
}

impl<T: CheckSame> Checker for Parentless<T> {}

impl<T: CheckSame> Prove<Parentless<T>> for Parentless<T> {
    type ArgumentError = T::Error;
    type ProofChainError = Infallible;
    type ParentsError = Infallible;

    fn check(&self, proof: &Parentless<T>) -> Outcome<T::Error, Infallible, Infallible> {
        match proof {
            Parentless::Any => Outcome::Proven,
            Parentless::This(that) => match self {
                Parentless::Any => Outcome::Proven,
                Parentless::This(this) => match this.check_same(that) {
                    Ok(()) => Outcome::Proven,
                    Err(e) => Outcome::ArgumentEscelation(e),
                },
            },
        }
    }
}
