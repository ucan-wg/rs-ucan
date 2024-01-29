use crate::{
    ability::traits::Command,
    proof::{checkable::Checkable, parentful::Parentful, parents::CheckParents, same::CheckSame},
};
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};
use url::Url;

use super::any as msg;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Send {
    pub to: Url,
    pub from: Url,
    pub message: String,
}

impl Command for Send {
    const COMMAND: &'static str = "msg/send";
}

impl From<Send> for Ipld {
    fn from(send: Send) -> Self {
        send.into()
    }
}

impl TryFrom<Ipld> for Send {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct SendBuilder {
    pub to: Option<Url>,
    pub from: Option<Url>,
    pub message: Option<String>,
}

impl From<SendBuilder> for Ipld {
    fn from(send: SendBuilder) -> Self {
        send.into()
    }
}

impl TryFrom<Ipld> for SendBuilder {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl Checkable for SendBuilder {
    type Heirarchy = Parentful<SendBuilder>;
}

impl CheckSame for SendBuilder {
    type Error = (); // FIXME better error
    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        self.to.check_same(&proof.to).map_err(|_| ())?; // FIXME
        self.from.check_same(&proof.from).map_err(|_| ())?;
        self.message.check_same(&proof.message).map_err(|_| ())
    }
}

impl CheckParents for SendBuilder {
    type Parents = msg::Any;
    type ParentError = <msg::Any as CheckSame>::Error;

    // FIXME rename other to proof
    fn check_parents(&self, other: &Self::Parents) -> Result<(), Self::ParentError> {
        self.from.check_same(&other.from).map_err(|_| ())
    }
}
