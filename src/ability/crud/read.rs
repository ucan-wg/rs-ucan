use super::any;
use crate::{
    ability::traits::Command,
    proof::{checkable::Checkable, parentful::Parentful, parents::CheckParents, same::CheckSame},
};
use libipld_core::{ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use url::Url;

// Read is its own builder
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Read {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub uri: Option<Url>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<BTreeMap<String, Ipld>>,
}

impl Command for Read {
    const COMMAND: &'static str = "crud/read";
}

impl From<Read> for Ipld {
    fn from(read: Read) -> Self {
        read.into()
    }
}

impl TryFrom<Ipld> for Read {
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld).map_err(|_| ())
    }
}

impl Checkable for Read {
    type Hierarchy = Parentful<Read>;
}

impl CheckSame for Read {
    type Error = ();
    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl CheckParents for Read {
    type Parents = any::Builder;
    type ParentError = ();
    fn check_parents(&self, other: &Self::Parents) -> Result<(), Self::ParentError> {
        Ok(())
    }
}
