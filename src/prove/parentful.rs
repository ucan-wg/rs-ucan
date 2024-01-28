use super::{
    internal::IsChecker,
    traits::{CheckParents, CheckSelf, Prove},
};

pub enum Parentful<T: CheckParents> {
    Any,
    Parents(T::Parents),
    Me(T),
}

// TODO better names & derivations
pub enum ParentfulError<T: CheckParents>
where
    T::Parents: CheckSelf,
{
    ParentError(T::ParentError),
    ParentSelfError(<<T as CheckParents>::Parents as CheckSelf>::SelfError),
    SelfError(<T as CheckSelf>::SelfError),

    // Compared self to parents
    EscelationError,
}

impl<T: CheckParents> IsChecker for Parentful<T> {}

impl<T: CheckParents> CheckSelf for Parentful<T>
where
    T::Parents: CheckSelf,
{
    type SelfError = ParentfulError<T>;

    fn check_against_self(&self, other: &Self) -> Result<(), Self::SelfError> {
        match self {
            Parentful::Any => Ok(()),
            Parentful::Parents(parents) => match other {
                Parentful::Any => Ok(()),
                Parentful::Parents(other_parents) => parents
                    .check_against_self(other_parents)
                    .map_err(ParentfulError::ParentSelfError),
                Parentful::Me(_other_me) => Err(ParentfulError::EscelationError),
            },
            Parentful::Me(me) => match other {
                Parentful::Any => Ok(()),
                Parentful::Parents(other_parents) => me
                    .check_against_parents(other_parents)
                    .map_err(ParentfulError::ParentError),
                Parentful::Me(other_me) => me
                    .check_against_self(other_me)
                    .map_err(ParentfulError::SelfError),
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
            Parentful::Me(me) => me
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
    fn check<'a>(&'a self, other: &'a Parentful<T>) -> Result<(), Self::ProveError> {
        match other {
            Parentful::Any => Ok(()),
            Parentful::Parents(parents) => self
                .check_against_parents(parents)
                .map_err(ParentfulError::ParentError),
            Parentful::Me(me) => self
                .check_against_self(&me)
                .map_err(ParentfulError::SelfError),
        }
    }
}
