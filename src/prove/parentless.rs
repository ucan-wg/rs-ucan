use super::{
    internal::IsChecker,
    traits::{CheckSelf, Prove},
};

pub enum Parentless<T> {
    Any,
    Me(T),
}

impl<T: CheckSelf> IsChecker for Parentless<T> {}

impl<T: CheckSelf> CheckSelf for Parentless<T> {
    type Error = T::Error;

    fn check_against_self(&self, other: &Self) -> Result<(), Self::Error> {
        match self {
            Parentless::Any => Ok(()), // FIXME MUST forward that this was an ANY this into the result!
            Parentless::Me(me) => match other {
                Parentless::Any => Ok(()),
                Parentless::Me(other) => me.check_against_self(other),
            },
        }
    }
}

impl<T: CheckSelf> Prove<Parentless<T>> for T {
    type ProveError = T::Error;
    fn check<'a>(&'a self, other: &'a Parentless<T>) -> Result<(), T::Error> {
        match other {
            Parentless::Any => Ok(()),
            Parentless::Me(me) => self.check_against_self(&me),
        }
    }
}
