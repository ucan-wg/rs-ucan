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
pub struct Receive {
    pub from: Option<Url>,
}

// #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
// #[serde(deny_unknown_fields)]
// pub struct MsgReceiveDeferrable {
//     to: Deferrable<Url>,
//     from: Deferrable<Url>,
// }

impl Command for Receive {
    const COMMAND: &'static str = "msg/send";
}

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

impl Checkable for Receive {
    type CheckAs = Parentful<Receive>;
}

impl CheckSelf for Receive {
    type Error = (); // FIXME better error
    fn check_against_self(&self, proof: &Self) -> Result<(), Self::Error> {
        if let Some(self_from) = &self.from {
            if let Some(proof_from) = &proof.from {
                if self_from != proof_from {
                    return Err(());
                }
            }
        }

        Ok(())
    }
}

impl CheckParents for Receive {
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
