use crate::prove::{
    parentful::Parentful,
    traits::{CheckParents, CheckSelf, HasChecker},
};
use libipld_core::ipld::Ipld;
use std::collections::BTreeMap;
use url::Url;

use super::any::AnyBuilder;

pub struct Read {
    pub uri: Url,
    pub args: BTreeMap<String, Ipld>,
}

pub struct ReadBuilder {
    pub uri: Option<Url>,
    pub args: BTreeMap<String, Ipld>,
}

impl HasChecker for ReadBuilder {
    type CheckAs = Parentful<ReadBuilder>;
}

impl CheckSelf for ReadBuilder {
    type SelfError = ();
    fn check_against_self(&self, other: &Self) -> Result<(), Self::SelfError> {
        Ok(())
    }
}

impl CheckParents for ReadBuilder {
    type Parents = AnyBuilder;
    type ParentError = ();
    fn check_against_parents(&self, other: &Self::Parents) -> Result<(), Self::ParentError> {
        Ok(())
    }
}
