use crate::prove::{
    parentful::Parentful,
    traits::{CheckParents, CheckSelf, HasChecker},
};
use url::Url;

use super::any::AnyBuilder;

pub struct MutateBuilder {
    pub uri: Option<Url>,
}

impl HasChecker for MutateBuilder {
    type CheckAs = Parentful<MutateBuilder>;
}

impl CheckSelf for MutateBuilder {
    type SelfError = ();
    fn check_against_self(&self, _other: &Self) -> Result<(), Self::SelfError> {
        Ok(())
    }
}

// TODO note to self, this is effectively a partial order
impl CheckParents for MutateBuilder {
    type Parents = AnyBuilder;
    type ParentError = ();

    fn check_against_parents(&self, other: &Self::Parents) -> Result<(), Self::ParentError> {
        Ok(())
    }
}
