use crate::{
    ability::traits::Command,
    prove::{
        parentful::Parentful,
        traits::{CheckParents, CheckSelf, Checkable},
    },
};
use libipld_core::{ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};
use url::Url;

use super::parents::Mutable;

// Destroy is its own builder
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Destroy {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<Url>,
}

impl Command for Destroy {
    const COMMAND: &'static str = "crud/destroy";
}

impl From<Destroy> for Ipld {
    fn from(destroy: Destroy) -> Self {
        destroy.into()
    }
}

impl TryFrom<Ipld> for Destroy {
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld).map_err(|_| ())
    }
}

impl Checkable for Destroy {
    type CheckAs = Parentful<Destroy>;
}

impl CheckSelf for Destroy {
    type Error = ();
    fn check_against_self(&self, _proof: &Self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl CheckParents for Destroy {
    type Parents = Mutable;
    type ParentError = ();

    fn check_against_parents(&self, other: &Self::Parents) -> Result<(), Self::ParentError> {
        match other {
            Mutable::Mutate(mutate) => Ok(()),
            Mutable::Any(any) => Ok(()),
        }
    }
}
