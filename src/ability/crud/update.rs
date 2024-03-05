//! Update existing resources.

use crate::{
    ability::{arguments, command::Command},
    invocation::{promise, promise::Resolves},
    ipld,
};
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

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
    path: Option<promise::Resolves<PathBuf>>,

    /// Optional arguments to be passed in the update.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    args: Option<promise::Resolves<arguments::Named<ipld::Promised>>>,
}

const COMMAND: &'static str = "/crud/update";

impl Command for Update {
    const COMMAND: &'static str = COMMAND;
}

impl Command for PromisedUpdate {
    const COMMAND: &'static str = COMMAND;
}

impl TryFrom<arguments::Named<ipld::Promised>> for PromisedUpdate {
    type Error = ();

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
                        Ipld::String(s) => path = Some(promise::Resolves::new(PathBuf::from(s))),
                        _ => return Err(()),
                    },
                },

                "args" => match prom {
                    ipld::Promised::Map(map) => {
                        args = Some(promise::Resolves::new(arguments::Named(map)))
                    }
                    ipld::Promised::WaitOk(cid) => {
                        args = Some(promise::Resolves::new(arguments::Named::new()));
                    }
                    ipld::Promised::WaitErr(cid) => {
                        args = Some(promise::Resolves::new(arguments::Named::new()));
                    }
                    ipld::Promised::WaitAny(cid) => {
                        args = Some(promise::Resolves::new(arguments::Named::new()));
                    }
                    _ => return Err(()),
                },

                _ => return Err(()),
            }
        }

        Ok(PromisedUpdate { path, args })
    }
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

impl From<Update> for Ipld {
    fn from(create: Update) -> Self {
        create.into()
    }
}

impl TryFrom<Ipld> for Update {
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        if let Ipld::Map(map) = ipld {
            if map.len() > 2 {
                return Err(()); // FIXME
            }

            Ok(Update {
                path: map
                    .get("path")
                    .map(|ipld| (ipld::Newtype(ipld.clone())).try_into().map_err(|_| ()))
                    .transpose()?,

                args: map
                    .get("args")
                    .map(|ipld| ipld.clone().try_into().map_err(|_| ()))
                    .transpose()?,
            })
        } else {
            Err(()) // FIXME
        }
    }
}

impl From<PromisedUpdate> for arguments::Named<Ipld> {
    fn from(promised: PromisedUpdate) -> Self {
        let mut named = arguments::Named::new();

        if let Some(path_res) = promised.path {
            named.insert(
                "path".to_string(),
                path_res.map(|p| ipld::Newtype::from(p).0).into(),
            );
        }

        if let Some(args_res) = promised.args {
            named.insert(
                "args".to_string(),
                args_res
                    .try_resolve()
                    .expect("FIXME")
                    .iter()
                    .try_fold(BTreeMap::new(), |mut map, (k, v)| {
                        map.insert(k.clone(), Ipld::try_from(v.clone()).ok()?); // FIXME double check
                        Some(map)
                    })
                    .expect("FIXME")
                    .into(),
            );
        }

        named
    }
}

impl From<Update> for PromisedUpdate {
    fn from(r: Update) -> PromisedUpdate {
        PromisedUpdate {
            path: r
                .path
                .map(|inner_path| promise::PromiseOk::Fulfilled(inner_path).into()),

            args: r.args.map(|inner_args| Resolves::new(inner_args.into())),
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
            named.insert("path".to_string(), path.into());
        }

        if let Some(args) = promised.args {
            named.insert("args".to_string(), args.into());
        }

        named
    }
}
