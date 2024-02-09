use super::parents::MutableParents;
use crate::{
    ability::{arguments, command::Command},
    invocation::promise,
    ipld::promised::PromisedIpld,
    proof::{checkable::Checkable, parentful::Parentful, parents::CheckParents, same::CheckSame},
};
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ready {
    /// Optional path within the resource.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,

    /// Additional arugments to pass in the request.
    pub args: arguments::Named<Ipld>,
}

impl From<Ready> for Ipld {
    fn from(udpdate: Ready) -> Self {
        udpdate.into()
    }
}

impl TryFrom<Ipld> for Ready {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl Command for Ready {
    const COMMAND: &'static str = "crud/update";
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Builder {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<BTreeMap<String, Ipld>>, // FIXME use a type param?
}

impl From<Builder> for Ipld {
    fn from(udpdate: Builder) -> Self {
        udpdate.into()
    }
}

impl TryFrom<Ipld> for Builder {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl Checkable for Builder {
    type Hierarchy = Parentful<Builder>;
}

impl CheckSame for Builder {
    type Error = (); // FIXME

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        self.path.check_same(&proof.path).map_err(|_| ())?;
        self.args.check_same(&proof.args).map_err(|_| ())
    }
}

impl CheckParents for Builder {
    type Parents = MutableParents;
    type ParentError = (); // FIXME

    fn check_parent(&self, proof: &Self::Parents) -> Result<(), Self::ParentError> {
        match proof {
            MutableParents::Any(any) => self.path.check_same(&any.path).map_err(|_| ()),
            MutableParents::Mutate(mutate) => self.path.check_same(&mutate.path).map_err(|_| ()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Promised {
    #[serde(skip_serializing_if = "promise::Resolves::resolved_none")]
    pub path: promise::Resolves<Option<PathBuf>>,

    pub args: promise::Resolves<arguments::Named<PromisedIpld>>,
}
