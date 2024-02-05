use crate::{
    ability::command::Command,
    proof::{checkable::Checkable, parentful::Parentful, parents::CheckParents, same::CheckSame},
};
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};
use url::Url;

use super::any as msg;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receive {
    pub from: Option<Url>,
}

impl Command for Receive {
    const COMMAND: &'static str = "msg/send";
}

impl Checkable for Receive {
    type Hierarchy = Parentful<Receive>;
}

impl CheckSame for Receive {
    type Error = (); // FIXME better error
    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        self.from.check_same(&proof.from).map_err(|_| ())
    }
}

impl CheckParents for Receive {
    type Parents = msg::Any;
    type ParentError = <msg::Any as CheckSame>::Error;

    fn check_parent(&self, proof: &Self::Parents) -> Result<(), Self::ParentError> {
        self.from.check_same(&proof.from).map_err(|_| ())
    }
}

////////////

impl From<Receive> for Ipld {
    fn from(receive: Receive) -> Self {
        receive.into()
    }
}

impl TryFrom<Ipld> for Receive {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}
