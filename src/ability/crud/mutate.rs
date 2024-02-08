use crate::{
    ability::command::Command,
    proof::{checkable::Checkable, parentful::Parentful, parents::CheckParents, same::CheckSame},
};
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};
use url::Url;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mutate {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<Url>,
}

impl Command for Mutate {
    const COMMAND: &'static str = "crud/mutate";
}

impl From<Mutate> for Ipld {
    fn from(mutate: Mutate) -> Self {
        mutate.into()
    }
}

impl TryFrom<Ipld> for Mutate {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl Checkable for Mutate {
    type Hierarchy = Parentful<Mutate>;
}

impl CheckSame for Mutate {
    type Error = ();
    fn check_same(&self, _proof: &Self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl CheckParents for Mutate {
    type Parents = super::Any;
    type ParentError = (); // FIXME

    fn check_parent(&self, _proof: &Self::Parents) -> Result<(), Self::ParentError> {
        Ok(())
    }
}
