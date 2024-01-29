use super::{
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

impl<T: CheckSame> Checker for Parentless<T> {}

impl<T: CheckSame> Prove<Parentless<T>> for T {
    type ArgumentError = T::Error;
    type ProofChainError = Infallible;

    fn check<'a>(&'a self, proof: &'a Parentless<T>) -> Outcome<T::Error, Infallible> {
        match proof {
            Parentless::Any => Outcome::Proven,
            Parentless::This(this) => match self.check_same(&this) {
                Ok(()) => Outcome::Proven,
                Err(e) => Outcome::ArgumentEscelation(e),
            },
        }
    }
}
