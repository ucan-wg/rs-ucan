//! The ability to receive messages

use crate::{
    ability::{arguments, command::Command},
    invocation::{promise, Resolvable},
    proof::{checkable::Checkable, parentful::Parentful, parents::CheckParents, same::CheckSame},
    url,
};
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};

#[cfg_attr(doc, aquamarine::aquamarine)]
/// The ability to receive messages
///
/// This ability is used to receive messages from other actors.
///
/// # Delegation Hierarchy
///
/// The hierarchy of message abilities is as follows:
///
/// ```mermaid
/// flowchart TB
///     top("*")
///
///     subgraph Message Abilities
///       any("msg/*")
///
///       subgraph Invokable
///         rec("msg/receive")
///       end
///     end
///
///     recrun{{"invoke"}}
///
///     top --> any
///     any --> rec -.-> recrun
///
///     style rec stroke:orange;
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Receive {
    /// An *optional* URL (e.g. email, DID, socket) to receive messages from.
    /// This assumes that the `subject` has the authority to issue such a capability.
    pub from: Option<url::Newtype>,
}

// FIXME needs promisory version

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
    type Parents = super::Any;
    type ParentError = <super::Any as CheckSame>::Error;

    fn check_parent(&self, proof: &Self::Parents) -> Result<(), Self::ParentError> {
        // self.from.check_same(&proof.from).map_err(|_| ())
        todo!() // FIXME
    }
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

#[derive(Debug, Clone, PartialEq)]
pub struct Promised {
    pub from: promise::Resolves<Option<url::Newtype>>,
}

impl From<Promised> for arguments::Named<Ipld> {
    fn from(promised: Promised) -> Self {
        let mut args = arguments::Named::new();

        match promised.from {
            promise::Resolves::Ok(from) => {
                args.insert("from".into(), from.into());
            }
            promise::Resolves::Err(from) => {
                args.insert("from".into(), from.into());
            }
        }

        args
    }
}

impl Resolvable for Receive {
    type Promised = Promised;

    fn try_resolve(p: Promised) -> Result<Self, Self::Promised> {
        promise::Resolves::try_resolve(p.from)
            .map(|from| Receive { from })
            .map_err(|from| Promised { from })
    }
}
