//! "Any" message ability (superclass of all message abilities)

use crate::{
    ability::{arguments, command::Command},
    proof::{parentless::NoParents, same::CheckSame},
};
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};
use url::Url;

#[cfg_attr(doc, aquamarine::aquamarine)]
/// The [`msg::Any`][Any] ability may not be invoked, but it is the superclass of
/// all other message abilities.
///
/// For example, the [`msg::Receive`][super::receive::Receive] ability may
/// be proven by the [`msg::Any`][Any] ability in a delegation chain.
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
///         send("msg/send")
///         rec("msg/receive")
///       end
///     end
///
///     sendrun{{"invoke"}}
///     recrun{{"invoke"}}
///
///     top --> any
///     any --> send -.-> sendrun
///     any --> rec -.-> recrun
///
///     style any stroke:orange;
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Any {
    pub from: Option<Url>,
}

impl Command for Any {
    const COMMAND: &'static str = "msg/*";
}

impl From<Any> for Ipld {
    fn from(any: Any) -> Self {
        any.into()
    }
}

impl TryFrom<Ipld> for Any {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl NoParents for Any {}

impl CheckSame for Any {
    type Error = ();
    fn check_same(&self, _proof: &Self) -> Result<(), Self::Error> {
        Ok(())
    }
}

impl From<Any> for arguments::Named<Ipld> {
    fn from(any: Any) -> arguments::Named<Ipld> {
        let mut args = arguments::Named::new();

        if let Some(from) = any.from {
            args.insert("from".into(), from.to_string().into());
        }

        args
    }
}
