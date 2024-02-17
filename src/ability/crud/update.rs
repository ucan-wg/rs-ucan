//! Update existing resources.
use super::parents::MutableParents;
use crate::{
    ability::{arguments, command::Command},
    delegation::Delegable,
    invocation::{promise, promise::Resolves, Resolvable},
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
    args: Option<arguments::Promised>,
}

impl Command for Ready {
    const COMMAND: &'static str = "crud/update";
}

impl Delegable for Ready {
    type Builder = Builder;
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

impl Resolvable for Ready {
    type Promised = Promised;

    fn try_resolve(p: Promised) -> Result<Ready, Promised> {
        // FIXME extract & cleanup
        let path = match p.path {
            Some(ref res_path) => match res_path.clone().try_resolve() {
                Ok(path) => Some(Ok(path)),
                Err(unresolved) => Some(Err(Promised {
                    path: Some(unresolved),
                    args: p.args.clone(),
                })),
            },
            None => None,
        }
        .transpose()?;

        // FIXME extract & cleanup
        let args = match p.args {
            Some(ref res_args) => match res_args.clone().try_resolve() {
                Ok(args) => {
                    let ipld = args.try_into().map_err(|_| p.clone())?;
                    Some(Ok(ipld))
                }
                Err(unresolved) => Some(Err(Promised {
                    path: path.clone().map(|p| Resolves::new(p)),
                    args: Some(unresolved),
                })),
            },
            None => None,
        }
        .transpose()?;

        Ok(Ready { path, args })
    }
}

impl From<Promised> for Builder {
    fn from(promised: Promised) -> Self {
        Builder {
            path: promised.path.and_then(|p| p.try_resolve().ok()),
            args: promised.args.and_then(|a| a.try_resolve_option()),
        }
    }
}
