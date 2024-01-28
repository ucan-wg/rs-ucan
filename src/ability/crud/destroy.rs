use crate::prove::{
    parentful::Parentful,
    traits::{CheckParents, CheckSelf, HasChecker},
};
use url::Url;

use super::parents::Mutable;

#[derive(Debug, Clone, PartialEq)]
pub struct CrudDestroy {
    pub uri: Url,
}

pub struct DestroyBuilder {
    pub uri: Option<Url>,
}

impl HasChecker for DestroyBuilder {
    type CheckAs = Parentful<DestroyBuilder>;
}

impl CheckSelf for DestroyBuilder {
    type Error = ();
    fn check_against_self(&self, _other: &Self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl CheckParents for DestroyBuilder {
    type Parents = Mutable;
    type ParentError = ();

    fn check_against_parents(&self, other: &Self::Parents) -> Result<(), Self::ParentError> {
        match other {
            Mutable::Mutate(mutate) => Ok(()),
            Mutable::Any(any) => Ok(()),
        }
    }
}
