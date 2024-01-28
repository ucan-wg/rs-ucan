use crate::prove::{
    parentful::Parentful,
    traits::{CheckParents, CheckSelf, HasChecker},
};
use libipld_core::ipld::Ipld;
use std::collections::BTreeMap;
use url::Url;

use super::parents::Mutable;

pub struct Create {
    pub uri: Url,
    pub args: BTreeMap<String, String>,
}

pub struct CreateBuilder {
    pub uri: Option<Url>,
    pub args: BTreeMap<String, Ipld>,
}

impl HasChecker for CreateBuilder {
    type CheckAs = Parentful<CreateBuilder>;
}

impl CheckSelf for CreateBuilder {
    type SelfError = (); // FIXME better error
    fn check_against_self(&self, other: &Self) -> Result<(), Self::SelfError> {
        if self.uri == other.uri {
            Ok(())
        } else {
            Err(())
        }
    }
}

impl CheckParents for CreateBuilder {
    type Parents = Mutable;
    type ParentError = ();

    fn check_against_parents(&self, other: &Self::Parents) -> Result<(), Self::ParentError> {
        match other {
            Mutable::Mutate(mutate) => Ok(()),
            Mutable::Any(any) => Ok(()),
        }
    }
}
