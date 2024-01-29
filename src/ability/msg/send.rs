use crate::{
    ability::traits::Command,
    prove::{
        parentful::Parentful,
        traits::{CheckParents, CheckSelf, Checkable},
    },
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
    type CheckAs = Parentful<SendBuilder>;
}

impl CheckSelf for SendBuilder {
    type Error = (); // FIXME better error
    fn check_against_self(&self, proof: &Self) -> Result<(), Self::Error> {
        if let Some(self_to) = &self.to {
            if let Some(proof_to) = &proof.to {
                if self_to != proof_to {
                    return Err(());
                }
            }
        }

        if let Some(self_from) = &self.from {
            if let Some(proof_from) = &proof.from {
                if self_from != proof_from {
                    return Err(());
                }
            }
        }

        if let Some(self_msg) = &self.message {
            if let Some(proof_msg) = &proof.message {
                if self_msg != proof_msg {
                    return Err(());
                }
            }
        }

        Ok(())
    }
}

impl CheckParents for SendBuilder {
    type Parents = msg::Any;
    type ParentError = <msg::Any as CheckSelf>::Error;

    // FIXME rename other to proof
    fn check_against_parents(&self, other: &Self::Parents) -> Result<(), Self::ParentError> {
        if let Some(self_from) = &self.from {
            if let Some(proof_from) = &other.from {
                if self_from != proof_from {
                    return Err(());
                }
            }
        }

        Ok(())
    }
}
