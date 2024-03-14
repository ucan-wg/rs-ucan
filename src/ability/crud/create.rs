//! Create new resources.

use crate::{
    ability::{arguments, command::Command},
    invocation::promise,
    ipld,
};
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use thiserror::Error;

#[cfg_attr(doc, aquamarine::aquamarine)]
/// The executable/dispatchable variant of the `crud/create` ability.
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
///       subgraph CRUD Abilities
///         any("crud/*")
///
///         mutate("crud/mutate")
///
///         subgraph Invokable
///           create("crud/create")
///         end
///       end
///     end
///
///     createpromise("crud::create::PromisedCreate")
///     createready("crud::create::Create")
///
///     top --> any --> mutate --> create
///     create -.->|invoke| createpromise -.->|resolve| createready -.-> exe{{execute}}
///
///     style createready stroke:orange;
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Create {
    /// An optional path to a sub-resource that is to be created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,

    /// Optional arguments for creation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<arguments::Named<Ipld>>,
}

impl From<Create> for Ipld {
    fn from(create: Create) -> Self {
        let mut map = BTreeMap::new();

        if let Some(path) = create.path {
            map.insert("path".to_string(), path.display().to_string().into());
        }

        if let Some(args) = create.args {
            map.insert("args".to_string(), args.into());
        }

        Ipld::Map(map)
    }
}

#[cfg_attr(doc, aquamarine::aquamarine)]
/// An invoked `crud/create` ability (but possibly awaiting another
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
///         mutate("crud/mutate")
///
///         subgraph Invokable
///           create("crud/create")
///         end
///       end
///     end
///
///     createpromise("crud::create::PromisedCreate")
///     createready("crud::create::Create")
///
///     top --> any --> mutate --> create
///     create -.->|invoke| createpromise -.->|resolve| createready -.-> exe{{execute}}
///
///     style createpromise stroke:orange;
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PromisedCreate {
    /// An optional path to a sub-resource that is to be created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<promise::Any<PathBuf>>,

    /// Optional arguments for creation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<promise::Any<arguments::Named<ipld::Promised>>>,
}

const COMMAND: &str = "/crud/create";

impl Command for Create {
    const COMMAND: &'static str = COMMAND;
}

impl Command for PromisedCreate {
    const COMMAND: &'static str = COMMAND;
}

impl TryFrom<arguments::Named<ipld::Promised>> for PromisedCreate {
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
                        ipld::Promised::WaitOk(cid) => Some(promise::Any::PendingOk(cid)),
                        ipld::Promised::WaitErr(cid) => Some(promise::Any::PendingErr(cid)),
                        ipld::Promised::WaitAny(cid) => Some(promise::Any::PendingAny(cid)),
                        _ => return Err(FromPromisedArgsError::InvalidArgs(prom)),
                    }
                }
                _ => return Err(FromPromisedArgsError::InvalidMapKey(k)),
            }
        }

        Ok(PromisedCreate { path, args })
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

impl TryFrom<arguments::Named<Ipld>> for Create {
    type Error = ();

    fn try_from(arguments: arguments::Named<Ipld>) -> Result<Self, Self::Error> {
        let mut path = None;
        let mut args = None;

        for (k, ipld) in arguments {
            match k.as_str() {
                "path" => {
                    if let Ipld::String(s) = ipld {
                        path = Some(PathBuf::from(s));
                    } else {
                        return Err(());
                    }
                }
                "args" => {
                    args = Some(ipld.try_into().map_err(|_| ())?);
                }
                _ => return Err(()),
            }
        }

        Ok(Create { path, args })
    }
}

impl From<Create> for PromisedCreate {
    fn from(r: Create) -> PromisedCreate {
        PromisedCreate {
            path: r.path.map(|inner_path| promise::Any::Resolved(inner_path)),

            args: r
                .args
                .map(|inner_args| promise::Any::Resolved(inner_args.into())),
        }
    }
}

impl promise::Resolvable for Create {
    type Promised = PromisedCreate;
}

impl From<Create> for arguments::Named<Ipld> {
    fn from(create: Create) -> Self {
        let mut named = arguments::Named::new();

        if let Some(path) = create.path {
            named.insert("path".to_string(), path.display().to_string().into());
        }

        if let Some(args) = create.args {
            named.insert("args".to_string(), args.into());
        }

        named
    }
}

impl From<PromisedCreate> for arguments::Named<ipld::Promised> {
    fn from(promised: PromisedCreate) -> Self {
        let mut named = arguments::Named::new();

        if let Some(path_prom) = promised.path {
            named.insert("path".to_string(), path_prom.to_promised_ipld());
        }

        if let Some(args_prom) = promised.args {
            named.insert("args".to_string(), args_prom.to_promised_ipld());
        }

        named
    }
}
