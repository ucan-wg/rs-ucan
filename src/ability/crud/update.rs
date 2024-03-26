//! Update existing resources.

use crate::{
    ability::{arguments, command::Command},
    invocation::promise,
    ipld,
};
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
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
///           update("crud/update")
///         end
///       end
///     end
///
///     updatepromise("crud::update::Promised")
///     updateready("crud::update::Update")
///
///     top --> any --> mutate --> update
///     update -.->|invoke| updatepromise -.->|resolve| updateready -.-> exe{{execute}}
///
///     style updateready stroke:orange;
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Update {
    /// An optional path to a sub-resource that is to be updated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    path: Option<PathBuf>,

    /// Optional arguments to be passed in the update.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    args: Option<arguments::Named<Ipld>>,
}

#[cfg_attr(doc, aquamarine::aquamarine)]
/// An invoked `crud/update` ability (but possibly awaiting another
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
///           update("crud/update")
///         end
///       end
///     end
///
///     updatepromise("crud::update::Promised")
///     updateready("crud::update::Update")
///
///     top --> any --> mutate --> update
///     update -.->|invoke| updatepromise -.->|resolve| updateready -.-> exe{{execute}}
///
///     style updatepromise stroke:orange;
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PromisedUpdate {
    /// An optional path to a sub-resource that is to be updated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    path: Option<promise::Any<PathBuf>>,

    /// Optional arguments to be passed in the update.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    args: Option<promise::Any<arguments::Named<ipld::Promised>>>,
}

const COMMAND: &'static str = "/crud/update";

impl Command for Update {
    const COMMAND: &'static str = COMMAND;
}

impl Command for PromisedUpdate {
    const COMMAND: &'static str = COMMAND;
}

impl TryFrom<arguments::Named<ipld::Promised>> for PromisedUpdate {
    type Error = FromPromisedArgsError;

    fn try_from(named: arguments::Named<ipld::Promised>) -> Result<Self, Self::Error> {
        let mut path = None;
        let mut args = None;

        for (key, prom) in named {
            match key.as_str() {
                "path" => match Ipld::try_from(prom) {
                    Err(pending) => {
                        path = Some(pending.into());
                    }
                    Ok(ipld) => match ipld {
                        Ipld::String(s) => path = Some(promise::Any::Resolved(PathBuf::from(s))),
                        other => return Err(FromPromisedArgsError::PathBodyNotAString(other)),
                    },
                },

                "args" => match prom {
                    ipld::Promised::Map(map) => {
                        args = Some(promise::Any::Resolved(arguments::Named(map)))
                    }
                    ipld::Promised::WaitOk(cid) => args = Some(promise::Any::PendingOk(cid)),
                    ipld::Promised::WaitErr(cid) => args = Some(promise::Any::PendingErr(cid)),
                    ipld::Promised::WaitAny(cid) => {
                        args = Some(promise::Any::PendingAny(cid));
                    }
                    _ => return Err(FromPromisedArgsError::InvalidArgs(prom)),
                },

                _ => return Err(FromPromisedArgsError::InvalidMapKey(key)),
            }
        }

        Ok(PromisedUpdate { path, args })
    }
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum FromPromisedArgsError {
    #[error("Path body is not a string")]
    PathBodyNotAString(Ipld),

    #[error("Invalid args {0}")]
    InvalidArgs(ipld::Promised),

    #[error("Invalid map key {0}")]
    InvalidMapKey(String),
}

impl TryFrom<arguments::Named<Ipld>> for Update {
    type Error = ();

    fn try_from(named: arguments::Named<Ipld>) -> Result<Self, Self::Error> {
        let mut path = None;
        let mut args = None;

        for (key, ipld) in named {
            match key.as_str() {
                "path" => {
                    if let Ipld::String(s) = ipld {
                        path = Some(PathBuf::from(s));
                    } else {
                        return Err(());
                    }
                }
                "args" => {
                    if let Ipld::Map(map) = ipld {
                        args = Some(arguments::Named(map));
                    } else {
                        return Err(());
                    }
                }
                _ => return Err(()),
            }
        }

        Ok(Update { path, args })
    }
}

impl From<Update> for arguments::Named<Ipld> {
    fn from(create: Update) -> Self {
        let mut named = arguments::Named::<Ipld>::new();

        if let Some(path) = create.path {
            named.insert("path".to_string(), Ipld::String(path.display().to_string()));
        }

        if let Some(args) = create.args {
            named.insert("args".to_string(), args.into());
        }

        named
    }
}

impl TryFrom<Ipld> for Update {
    type Error = TryFromIpldError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        if let Ipld::Map(map) = ipld {
            if map.len() > 2 {
                return Err(TryFromIpldError::TooManyKeys);
            }

            Ok(Update {
                path: map
                    .get("path")
                    .map(|ipld| {
                        (ipld::Newtype(ipld.clone()))
                            .try_into()
                            .map_err(TryFromIpldError::InvalidPath)
                    })
                    .transpose()?,

                args: map
                    .get("args")
                    .map(|ipld| {
                        arguments::Named::<Ipld>::try_from(ipld.clone())
                            .map_err(|_| TryFromIpldError::InvalidArgs)
                    })
                    .transpose()?,
            })
        } else {
            Err(TryFromIpldError::NotAMap)
        }
    }
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum TryFromIpldError {
    #[error("Not a map")]
    NotAMap,

    #[error("Too many keys")]
    TooManyKeys,

    #[error("Invalid path: {0}")]
    InvalidPath(ipld::newtype::NotAString),

    #[error("Invalid args: not a map")]
    InvalidArgs,
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum FromPromisedUpdateError {
    #[error("Unresolved args")]
    UnresolvedArgs(promise::Any<arguments::Named<ipld::Promised>>),

    #[error("Args pending")]
    ArgsPending(<Ipld as TryFrom<ipld::Promised>>::Error),

    #[error("Invalid map key {0}")]
    InvalidMapKey(String),
}

impl From<Update> for PromisedUpdate {
    fn from(r: Update) -> PromisedUpdate {
        PromisedUpdate {
            path: r.path.map(|inner_path| promise::Any::Resolved(inner_path)),

            args: r
                .args
                .map(|inner_args| promise::Any::Resolved(inner_args.into())),
        }
    }
}

impl promise::Resolvable for Update {
    type Promised = PromisedUpdate;
}

impl From<PromisedUpdate> for arguments::Named<ipld::Promised> {
    fn from(promised: PromisedUpdate) -> Self {
        let mut named = arguments::Named::new();

        if let Some(path) = promised.path {
            named.insert("path".to_string(), path.to_promised_ipld());
        }

        if let Some(args) = promised.args {
            named.insert("args".to_string(), args.to_promised_ipld());
        }

        named
    }
}
