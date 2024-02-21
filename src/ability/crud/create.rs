//! Create new resources.
use super::parents::MutableParents;
use crate::{
    ability::{arguments, command::Command},
    delegation::Delegable,
    invocation::{promise, promise::Resolves},
    ipld,
    proof::{checkable::Checkable, parentful::Parentful, parents::CheckParents, same::CheckSame},
};
use libipld_core::{cid::Cid, error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::Serialize;
use std::{collections::BTreeMap, path::PathBuf};

// FIXME deserialize instance

/// A helper for creating lifecycle instances of `crud/create` with the correct shape.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Generic<Path, Args> {
    /// An optional path to a sub-resource that is to be created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<Path>,

    /// Optional arguments for creation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<Args>,
}

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
///     createpromise("crud::create::Promised")
///     createready("crud::create::Ready")
///
///     top --> any --> mutate --> create
///     create -.->|invoke| createpromise -.->|resolve| createready -.-> exe{{execute}}
///
///     style createready stroke:orange;
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Ready {
    /// An optional path to a sub-resource that is to be created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,

    /// Optional arguments for creation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<arguments::Named<Ipld>>,
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
///     createpromise("crud::create::Promised")
///     createready("crud::create::Ready")
///
///     top --> any --> mutate --> create
///     create -.->|invoke| createpromise -.->|resolve| createready -.-> exe{{execute}}
///
///     style createpromise stroke:orange;
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Promised {
    /// An optional path to a sub-resource that is to be created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<promise::Resolves<PathBuf>>,

    /// Optional arguments for creation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<promise::Resolves<arguments::Named<ipld::Promised>>>,
}

const COMMAND: &str = "crud/create";

impl Command for Ready {
    const COMMAND: &'static str = COMMAND;
}

impl Command for Promised {
    const COMMAND: &'static str = COMMAND;
}

// impl TryFrom<Ipld> for Ready {
//     type Error = (); // FIXME
//
//     fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
//         if let Ipld::Map(mut map) = ipld {
//             if map.len() > 2 {
//                 return Err(()); // FIXME
//             }
//
//             Ok(Generic {
//                 path: map
//                     .remove("path")
//                     .map(|ipld| P::try_from(ipld).map_err(|_| ()))
//                     .transpose()?,
//
//                 args: map
//                     .remove("args")
//                     .map(|ipld| A::try_from(ipld).map_err(|_| ()))
//                     .transpose()?,
//             })
//         } else {
//             Err(()) // FIXME
//         }
//     }
// }

impl TryFrom<arguments::Named<Ipld>> for Ready {
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

        Ok(Ready { path, args })
    }
}

impl Delegable for Ready {
    type Builder = Ready;
}

impl Checkable for Ready {
    type Hierarchy = Parentful<Ready>;
}

impl CheckSame for Ready {
    type Error = (); // FIXME better error

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        if self.path == proof.path {
            Ok(())
        } else {
            Err(())
        }
    }
}

impl CheckParents for Ready {
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

// impl From<Promised> for arguments::Named<Ipld> {
//     fn from(promised: Promised) -> Self {
//         let mut named = arguments::Named::new();
//
//         if let Some(path) = promised.path {
//             named.insert("path".to_string(), Ipld::String(path.to_string()));
//         }
//
//         if let Some(args) = promised.args {
//             named.insert("args".to_string(), Ipld::from(args));
//         }
//
//         named
//     }
// }

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

// FIXME may want to name this something other than a TryFrom
//  impl From<Promised> for Builder {
//      fn from(promised: Promised) -> Self {
//          Builder {
//              path: promised.path.and_then(|x| x.try_resolve().ok()),
//              args: promised
//                  .args
//                  .and_then(|x| x.try_resolve().ok()?.try_into().ok()),
//          }
//      }
//  }

impl promise::Resolvable for Ready {
    type Promised = Promised;
}

impl From<Ready> for arguments::Named<Ipld> {
    fn from(builder: Ready) -> Self {
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
