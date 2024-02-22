//! Update existing resources.
use super::parents::MutableParents;
use crate::{
    ability::{arguments, command::Command},
    delegation::Delegable,
    invocation::{promise, promise::Resolves},
    ipld,
    proof::{checkable::Checkable, parentful::Parentful, parents::CheckParents, same::CheckSame},
};
use libipld_core::ipld::Ipld;
use serde::Serialize;
use std::{collections::BTreeMap, path::PathBuf};

// FIXME deserialize instance

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
///     updateready("crud::update::Ready")
///
///     top --> any --> mutate --> update
///     update -.->|invoke| updatepromise -.->|resolve| updateready -.-> exe{{execute}}
///
///     style updateready stroke:orange;
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Ready {
    /// An optional path to a sub-resource that is to be updated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    path: Option<PathBuf>,

    /// Optional arguments to be passed in the update.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    args: Option<arguments::Named<Ipld>>,
}

#[cfg_attr(doc, aquamarine::aquamarine)]
/// The delegatable ability for updating existing agents.
///
/// # Lifecycle
///
/// The lifecycle of a `crud/create` ability is as follows:
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
///     updateready("crud::update::Ready")
///
///     top --> any --> mutate --> update
///     update -.->|invoke| updatepromise -.->|resolve| updateready -.-> exe{{execute}}
///
///     style update stroke:orange;
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Builder {
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
///     updateready("crud::update::Ready")
///
///     top --> any --> mutate --> update
///     update -.->|invoke| updatepromise -.->|resolve| updateready -.-> exe{{execute}}
///
///     style updatepromise stroke:orange;
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Promised {
    /// An optional path to a sub-resource that is to be updated.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    path: Option<promise::Resolves<PathBuf>>,

    /// Optional arguments to be passed in the update.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    args: Option<promise::Resolves<arguments::Named<ipld::Promised>>>,
}

const COMMAND: &'static str = "crud/update";

impl Command for Ready {
    const COMMAND: &'static str = COMMAND;
}

impl Command for Builder {
    const COMMAND: &'static str = COMMAND;
}

impl Command for Promised {
    const COMMAND: &'static str = COMMAND;
}

impl TryFrom<arguments::Named<ipld::Promised>> for Promised {
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

        Ok(Promised { path, args })
    }
}

impl Delegable for Ready {
    type Builder = Builder;
}

impl TryFrom<arguments::Named<Ipld>> for Ready {
    type Error = ();

    fn try_from(named: arguments::Named<Ipld>) -> Result<Self, Self::Error> {
        Self::try_from_named(named).map_err(|_| ())
    }
}

impl TryFrom<arguments::Named<Ipld>> for Builder {
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

        Ok(Builder { path, args })
    }
}

impl From<Ready> for Builder {
    fn from(r: Ready) -> Self {
        Builder {
            path: r.path,
            args: r.args,
        }
    }
}

impl From<Builder> for Ready {
    fn from(builder: Builder) -> Self {
        Ready {
            path: builder.path,
            args: builder.args,
        }
    }
}

impl From<Ready> for Ipld {
    fn from(create: Ready) -> Self {
        create.into()
    }
}

impl TryFrom<Ipld> for Ready {
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        if let Ipld::Map(map) = ipld {
            if map.len() > 2 {
                return Err(()); // FIXME
            }

            Ok(Ready {
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

impl Checkable for Builder {
    type Hierarchy = Parentful<Builder>;
}

impl CheckSame for Builder {
    type Error = (); // FIXME better error

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        if self.path == proof.path {
            Ok(())
        } else {
            Err(())
        }
    }
}

impl CheckParents for Builder {
    type Parents = MutableParents;
    type ParentError = (); // FIXME

    fn check_parent(&self, other: &Self::Parents) -> Result<(), Self::ParentError> {
        if let Some(self_path) = &self.path {
            match other {
                MutableParents::Any(any) => {
                    // FIXME check the args, too!
                    if let Some(proof_path) = &any.path {
                        if self_path != proof_path {
                            return Err(());
                        }
                    }
                }
                MutableParents::Mutate(mutate) => {
                    // FIXME check the args, too!
                    if let Some(proof_path) = &mutate.path {
                        if self_path != proof_path {
                            return Err(());
                        }
                    }
                }
            }
        }

        Ok(())
    }
}

impl From<Promised> for arguments::Named<Ipld> {
    fn from(promised: Promised) -> Self {
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

impl From<Ready> for Promised {
    fn from(r: Ready) -> Promised {
        Promised {
            path: r
                .path
                .map(|inner_path| promise::PromiseOk::Fulfilled(inner_path).into()),

            args: r.args.map(|inner_args| Resolves::new(inner_args.into())),
        }
    }
}

impl promise::Resolvable for Ready {
    type Promised = Promised;
}

impl From<Promised> for Builder {
    fn from(promised: Promised) -> Self {
        Builder {
            path: promised.path.and_then(|p| p.try_resolve().ok()),
            args: promised.args.and_then(|a| a.try_resolve_option()),
        }
    }
}

impl From<Builder> for arguments::Named<Ipld> {
    fn from(builder: Builder) -> Self {
        let mut named = arguments::Named::new();

        if let Some(path) = builder.path {
            named.insert(
                "path".to_string(),
                path.into_os_string()
                    .into_string()
                    .expect("PathBuf to generate valid paths") // FIXME reasonable assumption?
                    .into(),
            );
        }

        if let Some(args) = builder.args {
            named.insert("args".to_string(), args.into());
        }

        named
    }
}

impl From<Promised> for arguments::Named<ipld::Promised> {
    fn from(promised: Promised) -> Self {
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
