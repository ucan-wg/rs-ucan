use crate::prove::{
    parentful::Parentful,
    traits::{CheckParents, CheckSelf, HasChecker},
};
use libipld_core::ipld::Ipld;
use std::collections::BTreeMap;
use url::Url;

use super::parents::Mutable;

#[derive(Debug, Clone, PartialEq)]
pub struct Update {
    pub uri: Url,
    pub args: BTreeMap<Box<str>, String>,
}

pub struct UpdateBuilder {
    pub uri: Option<Url>,
    pub args: BTreeMap<String, Ipld>, // FIXME use a type param?
}

impl HasChecker for UpdateBuilder {
    type CheckAs = Parentful<UpdateBuilder>;
}

impl CheckSelf for UpdateBuilder {
    type Error = ();
    fn check_against_self(&self, _other: &Self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl CheckParents for UpdateBuilder {
    type Parents = Mutable;
    type ParentError = ();

    fn check_against_parents(&self, other: &Self::Parents) -> Result<(), Self::ParentError> {
        match other {
            Mutable::Mutate(mutate) => Ok(()),
            Mutable::Any(any) => Ok(()),
        }
    }
}
