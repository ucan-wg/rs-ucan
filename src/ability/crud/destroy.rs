//! Destroy a resource.

// use super::parents::MutableParents;
use crate::{
    ability::{arguments, command::Command},
    // delegation::Delegable,
    invocation::promise,
    ipld,
    // proof::{checkable::Checkable, parentful::Parentful, parents::CheckParents, same::CheckSame},
};
use libipld_core::ipld::Ipld;
use serde::Serialize;
use std::path::PathBuf;

/// A helper for creating lifecycle instances of `crud/create` with the correct shape.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Generic<Path> {
    /// An optional path to a sub-resource that is to be destroyed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<Path>,
}

#[cfg_attr(doc, aquamarine::aquamarine)]
/// The executable/dispatchable variant of the `crud/destroy` ability.
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
///           destroy("crud/destroy")
///         end
///       end
///     end
///
///     destroypromise("crud::destroy::Promised")
///     destroyready("crud::destroy::Ready")
///
///     top --> any --> mutate --> destroy
///     destroy -.->|invoke| destroypromise -.->|resolve| destroyready -.-> exe{{execute}}
///
///     style destroyready stroke:orange;
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Ready {
    /// An optional path to a sub-resource that is to be destroyed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
}

#[cfg_attr(doc, aquamarine::aquamarine)]
/// An invoked `crud/destroy` ability (but possibly awaiting another
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
///           destroy("crud/destroy")
///         end
///       end
///     end
///
///     destroypromise("crud::destroy::Promised")
///     destroyready("crud::destroy::Ready")
///
///     top --> any --> mutate --> destroy
///     destroy -.->|invoke| destroypromise -.->|resolve| destroyready -.-> exe{{execute}}
///
///     style destroypromise stroke:orange;
/// ```
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Promised {
    /// An optional path to a sub-resource that is to be destroyed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<promise::Resolves<PathBuf>>,
}

impl TryFrom<arguments::Named<ipld::Promised>> for Promised {
    type Error = ();

    fn try_from(arguments: arguments::Named<ipld::Promised>) -> Result<Self, Self::Error> {
        let mut path = None;

        for (k, prom) in arguments {
            match k.as_str() {
                "path" => match prom {
                    ipld::Promised::String(s) => {
                        path = Some(promise::Resolves::Ok(
                            promise::PromiseOk::Fulfilled(PathBuf::from(s)).into(),
                        ));
                    }
                    ipld::Promised::WaitOk(cid) => {
                        path = Some(promise::PromiseOk::Pending(cid).into());
                    }
                    ipld::Promised::WaitErr(cid) => {
                        path = Some(promise::PromiseErr::Pending(cid).into());
                    }
                    ipld::Promised::WaitAny(cid) => {
                        todo!() // FIXME //  path = Some(promise::PromiseAny::Pending(cid).into());
                    }
                    _ => return Err(()),
                },
                _ => return Err(()),
            }
        }

        Ok(Promised { path })
    }
}

const COMMAND: &'static str = "/crud/destroy";

impl Command for Ready {
    const COMMAND: &'static str = COMMAND;
}

impl Command for Promised {
    const COMMAND: &'static str = COMMAND;
}

// impl Delegable for Ready {
//     type Builder = Ready;
// }

impl TryFrom<arguments::Named<Ipld>> for Ready {
    type Error = (); // FIXME

    fn try_from(args: arguments::Named<Ipld>) -> Result<Self, Self::Error> {
        let mut path = None;

        for (k, ipld) in args {
            match k.as_str() {
                "path" => {
                    if let Ipld::String(s) = ipld {
                        path = Some(PathBuf::from(s));
                    }
                }
                _ => return Err(()),
            }
        }

        Ok(Ready { path })
    }
}

// impl Checkable for Ready {
//     type Hierarchy = Parentful<Ready>;
// }
//
// impl CheckSame for Ready {
//     type Error = (); // FIXME better error
//
//     fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
//         if self.path == proof.path {
//             Ok(())
//         } else {
//             Err(())
//         }
//     }
// }
//
// impl CheckParents for Ready {
//     type Parents = MutableParents;
//     type ParentError = (); // FIXME
//
//     fn check_parent(&self, other: &Self::Parents) -> Result<(), Self::ParentError> {
//         if let Some(self_path) = &self.path {
//             match other {
//                 MutableParents::Any(any) => {
//                     if let Some(proof_path) = &any.path {
//                         if self_path != proof_path {
//                             return Err(());
//                         }
//                     }
//                 }
//                 MutableParents::Mutate(mutate) => {
//                     if let Some(proof_path) = &mutate.path {
//                         if self_path != proof_path {
//                             return Err(());
//                         }
//                     }
//                 }
//             }
//         }
//
//         Ok(())
//     }
// }

impl From<Promised> for arguments::Named<Ipld> {
    fn from(promised: Promised) -> Self {
        let mut named = arguments::Named::new();

        if let Some(path_res) = promised.path {
            named.insert(
                "path".to_string(),
                path_res.map(|p| ipld::Newtype::from(p).0).into(),
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
        }
    }
}

impl promise::Resolvable for Ready {
    type Promised = Promised;
}

impl From<Promised> for arguments::Named<ipld::Promised> {
    fn from(promised: Promised) -> Self {
        let mut named = arguments::Named::new();

        if let Some(path) = promised.path {
            named.insert("path".to_string(), path.into());
        }

        named
    }
}

impl From<Promised> for Ready {
    fn from(p: Promised) -> Ready {
        Ready {
            path: p
                .path
                .map(|inner_path| inner_path.try_resolve().ok())
                .flatten(),
        }
    }
}

impl From<Ready> for arguments::Named<Ipld> {
    fn from(ready: Ready) -> Self {
        let mut named = arguments::Named::new();

        if let Some(path) = ready.path {
            named.insert(
                "path".to_string(),
                path.into_os_string()
                    .into_string()
                    .expect("PathBuf to generate valid paths") // FIXME reasonable assumption?
                    .into(),
            );
        }

        named
    }
}
