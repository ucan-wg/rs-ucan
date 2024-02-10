//! Utilities for working with abilties that *don't* have a delegation hirarchy
//!
use super::{
    checkable::Checkable,
    internal::Checker,
    prove::{Prove, Success},
    same::CheckSame,
};
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{de::DeserializeOwned, Deserialize, Serialize};

/// The possible cases for an [ability][crate::ability]'s
/// [Delegation][crate::delegation::Delegation] chain when
/// it has no parent abilities (no hierarchy).
///
/// This type is generally not used directly, but rather is
/// called in the plumbing of the library.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Parentless<T> {
    /// The "top" ability (`*`)
    Any,

    /// The (invokable) ability itself.
    This(T),
}

// FIXME generally useful (e.g. checkiung `_/*`); move to its own module and rename?
/// Error cases when checking proofs
#[derive(Debug, Clone, PartialEq)]
pub enum ParentlessError<T: CheckSame> {
    /// The `cmd` field was more powerful than the proof.
    ///
    /// i.e. it behaves like moving "down" the delegation chain not "up"
    CommandEscelation,

    /// The `args` field was more powerful than the proof
    ArgumentEscelation(T::Error),
}

// FIXME better name
/// A helper trait to indicate that a type has no parents.
///
/// This behaves as an alias for `Checkable::<Hierarchy = Parentless<T>>`.
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

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        match proof {
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

impl<T: CheckSame> Prove for Parentless<T> {
    type Error = ParentlessError<T>;

    fn check(&self, proof: &Parentless<T>) -> Result<Success, Self::Error> {
        match proof {
            Parentless::Any => Ok(Success::ProvenByAny),
            Parentless::This(that) => match self {
                Parentless::Any => Ok(Success::Proven),
                Parentless::This(this) => match this.check_same(that) {
                    Ok(()) => Ok(Success::Proven),
                    Err(e) => Err(ParentlessError::ArgumentEscelation(e)),
                },
            },
        }
    }
}
