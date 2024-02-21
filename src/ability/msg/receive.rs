//! The ability to receive messages

use crate::{
    ability::{arguments, command::Command},
    delegation::Delegable,
    invocation::promise,
    ipld,
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

const COMMAND: &'static str = "msg/send";

impl Command for Receive {
    const COMMAND: &'static str = COMMAND;
}

impl Command for Promised {
    const COMMAND: &'static str = COMMAND;
}

impl Delegable for Receive {
    type Builder = Receive;
}

impl TryFrom<arguments::Named<Ipld>> for Receive {
    type Error = ();

    fn try_from(arguments: arguments::Named<Ipld>) -> Result<Self, Self::Error> {
        let mut from = None;

        for (key, ipld) in arguments {
            match key.as_str() {
                "from" => {
                    from = Some(url::Newtype::try_from(ipld).map_err(|_| ())?);
                }
                _ => return Err(()),
            }
        }

        Ok(Receive { from })
    }
}

impl From<Receive> for arguments::Named<Ipld> {
    fn from(receive: Receive) -> Self {
        let mut args = arguments::Named::new();

        if let Some(from) = receive.from {
            args.insert("from".into(), from.into());
        }

        args
    }
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
        if let Some(from) = &self.from {
            if let Some(proof_from) = &proof.from {
                if from != &url::Newtype(proof_from.clone()) {
                    return Err(());
                }
            }
        }

        Ok(())
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
    pub from: Option<promise::Resolves<Option<url::Newtype>>>,
}

impl From<Promised> for arguments::Named<Ipld> {
    fn from(promised: Promised) -> Self {
        let mut args = arguments::Named::new();

        if let Some(from) = promised.from {
            match from {
                promise::Resolves::Ok(from) => {
                    args.insert("from".into(), from.into());
                }
                promise::Resolves::Err(from) => {
                    args.insert("from".into(), from.into());
                }
            }
        }

        args
    }
}

impl promise::Resolvable for Receive {
    type Promised = Promised;
}

impl From<Promised> for arguments::Named<ipld::Promised> {
    fn from(promised: Promised) -> Self {
        let mut args = arguments::Named::new();

        if let Some(from) = promised.from {
            let _ = ipld::Promised::from(from).with_resolved(|ipld| {
                args.insert("from".into(), ipld.into());
            });
        }

        args
    }
}
