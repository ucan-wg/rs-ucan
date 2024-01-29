use super::{
    internal::Checker,
    traits::{CheckSelf, Prove},
};
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

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

impl<T: CheckSelf> Checker for Parentless<T> {}

impl<T: CheckSelf> CheckSelf for Parentless<T> {
    type Error = T::Error;

    fn check_against_self(&self, proof: &Self) -> Result<(), Self::Error> {
        match self {
            Parentless::Any => Ok(()), // FIXME MUST forward that this was an ANY this into the result!
            Parentless::This(this) => match proof {
                Parentless::Any => Ok(()),
                Parentless::This(other) => this.check_against_self(other),
            },
        }
    }
}

impl<T: CheckSelf> Prove<Parentless<T>> for T {
    type ProveError = T::Error;
    fn check<'a>(&'a self, proof: &'a Parentless<T>) -> Result<(), T::Error> {
        match proof {
            Parentless::Any => Ok(()),
            Parentless::This(this) => self.check_against_self(&this),
        }
    }
}
