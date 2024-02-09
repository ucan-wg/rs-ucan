use super::{error::ProofError, parents::MutableParents};
use crate::{
    ability::{arguments, command::Command},
    invocation::{promise, Resolvable},
    proof::{checkable::Checkable, parentful::Parentful, parents::CheckParents, same::CheckSame},
};
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

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
    pub args: Option<arguments::Named<Ipld>>,
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
    type Error = ProofError;

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        self.path
            .check_same(&proof.path)
            .map_err(Into::<ProofError>::into)?;
        self.args.check_same(&proof.args).map_err(Into::into)
    }
}

impl CheckParents for Builder {
    type Parents = MutableParents;
    type ParentError = ProofError;

    fn check_parent(&self, proof: &Self::Parents) -> Result<(), Self::ParentError> {
        match proof {
            MutableParents::Any(any) => self.path.check_same(&any.path).map_err(Into::into),
            MutableParents::Mutate(mutate) => {
                self.path.check_same(&mutate.path).map_err(Into::into)
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Promised {
    #[serde(skip_serializing_if = "promise::Resolves::resolved_none")]
    pub path: promise::Resolves<Option<PathBuf>>,

    pub args: arguments::Promised,
}

impl From<Ready> for Promised {
    fn from(r: Ready) -> Promised {
        Promised {
            path: promise::PromiseOk::Fulfilled(r.path).into(),
            args: promise::PromiseOk::Fulfilled(r.args.into()).into(),
        }
    }
}

impl From<Promised> for arguments::Named<Ipld> {
    fn from(p: Promised) -> arguments::Named<Ipld> {
        p.into()
    }
}

impl Resolvable for Ready {
    type Promised = Promised;

    fn try_resolve(p: Promised) -> Result<Ready, Promised> {
        // FIXME resolve2?
        //  FIXME lots of clone
        Ok(Ready {
            path: p.path.clone().try_resolve().map_err(|path| Promised {
                path,
                args: p.args.clone(),
            })?,

            args: p
                .args
                .clone()
                .try_into()
                .map_err(|args| Promised { path: p.path, args })?,
        })
    }
}
