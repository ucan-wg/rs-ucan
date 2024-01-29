use super::{
    internal::Checker,
    traits::{CheckParents, CheckSelf, Prove},
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

// TODO better names & derivations
pub enum ParentfulError<T: CheckParents>
where
    T::Parents: CheckSelf,
{
    ParentError(T::ParentError),
    // FIXME needs a WAAAAAY better name
    ParentSelfError(<<T as CheckParents>::Parents as CheckSelf>::Error),
    Error(<T as CheckSelf>::Error),

    // Compared self to parents
    EscelationError,
}

impl<T: CheckParents> Checker for Parentful<T> {}

impl<T: CheckParents> CheckSelf for Parentful<T>
where
    T::Parents: CheckSelf,
{
    type Error = ParentfulError<T>;

    fn check_against_self(&self, proof: &Self) -> Result<(), Self::Error> {
        match self {
            Parentful::Any => Ok(()),
            Parentful::Parents(parents) => match proof {
                Parentful::Any => Ok(()),
                Parentful::Parents(other_parents) => parents
                    .check_against_self(other_parents)
                    .map_err(ParentfulError::ParentSelfError),
                Parentful::This(_other_me) => Err(ParentfulError::EscelationError),
            },
            Parentful::This(this) => match proof {
                Parentful::Any => Ok(()),
                Parentful::Parents(other_parents) => this
                    .check_against_parents(other_parents)
                    .map_err(ParentfulError::ParentError),
                Parentful::This(that) => {
                    this.check_against_self(that).map_err(ParentfulError::Error)
                }
            },
        }
    }
}

impl<T: CheckSelf + CheckParents> CheckParents for Parentful<T>
where
    Parentful<T>: CheckSelf,
    T::Parents: CheckSelf,
{
    type Parents = T::Parents;
    type ParentError = ParentfulError<T>;

    fn check_against_parents(&self, other: &T::Parents) -> Result<(), Self::ParentError> {
        // FIXME note to self: see if you can extract the parentful stuff out into the to level Prove
        match self {
            Parentful::Any => Ok(()),
            Parentful::Parents(parents) => parents.check_against_self(other).map_err(|_| todo!()), // FIXME ParentfulError::ParentError),
            Parentful::This(this) => this
                .check_against_parents(other)
                .map_err(ParentfulError::ParentError),
        }
    }
}

impl<T: CheckParents> Prove<Parentful<T>> for T
where
    T::Parents: CheckSelf,
{
    type ProveError = ParentfulError<T>;
    fn check<'a>(&'a self, proof: &'a Parentful<T>) -> Result<(), Self::ProveError> {
        match proof {
            Parentful::Any => Ok(()),
            Parentful::Parents(parents) => self
                .check_against_parents(parents)
                .map_err(ParentfulError::ParentError),
            Parentful::This(that) => self
                .check_against_self(&that)
                .map_err(ParentfulError::Error),
        }
    }
}
