//! Read from a resource.

use crate::{
    ability::{arguments, command::Command},
    invocation::promise,
    ipld,
};
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use thiserror::Error;

#[cfg_attr(doc, aquamarine::aquamarine)]
/// This ability is used to fetch messages from other actors.
///
/// # Lifecycle
///
/// The relevant hierarchy of CRUD abilities is as follows:
///
/// ```mermaid
/// flowchart LR
///     subgraph Delegations
///       top("*")
///
///       any("crud/*")
///
///       subgraph Invokable
///         read("crud/read")
///       end
///     end
///
///     readpromise("crud::read::Promised")
///     readready("crud::read::Read")
///
///     top --> any --> read
///     read -.->|invoke| readpromise -.->|resolve| readready -.-> exe{{execute}}
///
///     style readready stroke:orange;
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Read {
    /// An optional path to a sub-resource that is to be read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,

    /// Optional arguments to modify the read request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<arguments::Named<Ipld>>,
}

impl From<Read> for Ipld {
    fn from(ready: Read) -> Self {
        let mut map = BTreeMap::new();

        if let Some(path) = ready.path {
            map.insert("path".to_string(), Ipld::String(path.display().to_string()));
        }

        if let Some(args) = ready.args {
            map.insert("args".to_string(), args.into());
        }

        map.into()
    }
}

#[cfg_attr(doc, aquamarine::aquamarine)]
/// An invoked `crud/read` ability (but possibly awaiting another
/// [`Invocation`][crate::invocation::Invocation]).
///
/// # Delegation Hierarchy
///
/// The hierarchy of CRUD abilities is as follows:
///
/// ```mermaid
/// flowchart LR
///     subgraph Delegations
///       top("*")
///
///       subgraph CRUD Abilities
///         any("crud/*")
///
///         subgraph Invokable
///           read("crud/read")
///         end
///       end
///     end
///
///     readpromise("crud::read::Promised")
///     readready("crud::read::Read")
///
///     top --> any --> read
///     read -.->|invoke| readpromise -.->|resolve| readready -.-> exe{{execute}}
///
///     style readpromise stroke:orange;
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PromisedRead {
    /// An optional path to a sub-resource that is to be read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<promise::Any<PathBuf>>,

    /// Optional arguments to modify the read request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<promise::Any<arguments::Named<ipld::Promised>>>,
}

impl TryFrom<arguments::Named<ipld::Promised>> for PromisedRead {
    type Error = FromPromisedArgsError;

    fn try_from(arguments: arguments::Named<ipld::Promised>) -> Result<Self, Self::Error> {
        let mut path = None;
        let mut args = None;

        for (k, prom) in arguments {
            match k.as_str() {
                "path" => match prom {
                    ipld::Promised::String(s) => {
                        path = Some(promise::Any::Resolved(PathBuf::from(s)).into());
                    }
                    ipld::Promised::WaitOk(cid) => {
                        path = Some(promise::Any::PendingOk(cid).into());
                    }
                    ipld::Promised::WaitErr(cid) => {
                        path = Some(promise::Any::PendingErr(cid).into());
                    }
                    ipld::Promised::WaitAny(cid) => {
                        path = Some(promise::Any::PendingAny(cid).into());
                    }
                    _ => return Err(FromPromisedArgsError::InvalidPath(k)),
                },

                "args" => {
                    args = match prom {
                        ipld::Promised::Map(map) => {
                            Some(promise::Any::Resolved(arguments::Named(map)).into())
                        }
                        ipld::Promised::WaitOk(cid) => Some(promise::Any::PendingOk(cid).into()),
                        ipld::Promised::WaitErr(cid) => Some(promise::Any::PendingErr(cid).into()),
                        ipld::Promised::WaitAny(cid) => Some(promise::Any::PendingAny(cid).into()),
                        _ => return Err(FromPromisedArgsError::InvalidArgs(prom)),
                    }
                }
                _ => return Err(FromPromisedArgsError::InvalidMapKey(k)),
            }
        }

        Ok(PromisedRead { path, args })
    }
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum FromPromisedArgsError {
    #[error("Invalid path {0}")]
    InvalidPath(String),

    #[error("Invalid args {0}")]
    InvalidArgs(ipld::Promised),

    #[error("Invalid map key {0}")]
    InvalidMapKey(String),
}

const COMMAND: &'static str = "/crud/read";

impl Command for Read {
    const COMMAND: &'static str = COMMAND;
}

impl Command for PromisedRead {
    const COMMAND: &'static str = COMMAND;
}

impl TryFrom<Ipld> for Read {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl From<Read> for arguments::Named<Ipld> {
    fn from(ready: Read) -> Self {
        let mut named = arguments::Named::new();

        if let Some(path) = ready.path {
            named.insert(
                "path".to_string(),
                path.into_os_string()
                    .into_string()
                    .expect("PathBuf should make a valid path")
                    .into(),
            );
        }

        if let Some(args) = ready.args {
            named.insert("args".to_string(), args.into());
        }

        named
    }
}

impl TryFrom<arguments::Named<Ipld>> for Read {
    type Error = ();

    fn try_from(arguments: arguments::Named<Ipld>) -> Result<Read, Self::Error> {
        let mut path = None;
        let mut args = None;

        for (k, v) in arguments.into_iter() {
            match k.as_str() {
                "path" => {
                    if let Ipld::String(string) = v {
                        path = Some(PathBuf::from(string));
                    } else {
                        return Err(());
                    }
                }
                "args" => {
                    args = Some(arguments::Named::try_from(v).map_err(|_| ())?);
                }
                _ => return Err(()),
            }
        }

        Ok(Read { path, args })
    }
}

impl promise::Resolvable for Read {
    type Promised = PromisedRead;
}

impl From<PromisedRead> for arguments::Named<ipld::Promised> {
    fn from(promised: PromisedRead) -> Self {
        let mut named = arguments::Named::new();

        if let Some(path_res) = promised.path {
            named.insert("path".to_string(), path_res.to_promised_ipld());
        }

        if let Some(args_res) = promised.args {
            named.insert("args".to_string(), args_res.to_promised_ipld());
        }

        named
    }
}
