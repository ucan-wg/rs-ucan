use crate::{
    ability::traits::Command,
    proof::{checkable::Checkable, parentful::Parentful, parents::CheckParents, same::CheckSame},
};
use libipld_core::{ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};
use url::Url;

use super::any::AnyBuilder;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MutateBuilder {
    pub uri: Option<Url>,
}

impl Command for MutateBuilder {
    const COMMAND: &'static str = "crud/mutate";
}

impl From<MutateBuilder> for Ipld {
    fn from(mutate: MutateBuilder) -> Self {
        mutate.into()
    }
}

impl TryFrom<Ipld> for MutateBuilder {
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld).map_err(|_| ())
    }
}

impl Checkable for MutateBuilder {
    type CheckAs = Parentful<MutateBuilder>;
}

impl CheckSame for MutateBuilder {
    type Error = ();
    fn check_same(&self, _proof: &Self) -> Result<(), Self::Error> {
        Ok(())
    }
}

// TODO note to self, this is effectively a partial order
impl CheckParents for MutateBuilder {
    type Parents = AnyBuilder;
    type ParentError = ();

    fn check_parents(&self, _proof: &Self::Parents) -> Result<(), Self::ParentError> {
        Ok(())
    }
}
