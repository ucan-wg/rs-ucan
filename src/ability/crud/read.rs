//! Read from a resource.

use super::any as crud;
use crate::{
    ability::{arguments, command::Command},
    delegation::Delegable,
    invocation::{promise, promise::Resolves, Resolvable},
    ipld,
    proof::{checkable::Checkable, parentful::Parentful, parents::CheckParents, same::CheckSame},
};
use libipld_core::ipld::Ipld;
use serde::Serialize;
use std::path::PathBuf;

// FIXME deserialize instance

/// A helper for creating lifecycle instances of `crud/create` with the correct shape.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Generic<Path, Args> {
    /// An optional path to a sub-resource that is to be read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<Path>,

    /// Optional arguments to modify the read request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<Args>,
}

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
///     readready("crud::read::Ready")
///
///     top --> any --> read
///     read -.->|invoke| readpromise -.->|resolve| readready -.-> exe{{execute}}
///
///     style readready stroke:orange;
/// ```
pub type Ready = Generic<PathBuf, arguments::Named<Ipld>>;

#[cfg_attr(doc, aquamarine::aquamarine)]
/// The delegatable ability for reading resources.
///
/// # Lifecycle
///
/// The lifecycle of a `crud/read` ability is as follows:
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
///     readready("crud::read::Ready")
///
///     top --> any --> read
///     read -.->|invoke| readpromise -.->|resolve| readready -.-> exe{{execute}}
///
///     style read stroke:orange;
/// ```
pub type Builder = Generic<PathBuf, arguments::Named<Ipld>>;

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
///     readready("crud::read::Ready")
///
///     top --> any --> read
///     read -.->|invoke| readpromise -.->|resolve| readready -.-> exe{{execute}}
///
///     style readpromise stroke:orange;
/// ```
pub type Promised = Generic<promise::Resolves<PathBuf>, arguments::Promised>;

impl<P, A> Command for Generic<P, A> {
    const COMMAND: &'static str = "crud/read";
}

impl Delegable for Ready {
    type Builder = Builder;
}

impl From<Promised> for Builder {
    fn from(promised: Promised) -> Self {
        Builder {
            path: promised.path.and_then(|p| p.try_resolve().ok()),
            args: promised.args.and_then(|p| p.try_resolve_option()), // FIXME this needs to read better
        }
    }
}

// FIXME resolves vs resolvable is confusing

impl<P: Into<Ipld>, A: Into<Ipld>> From<Generic<P, A>> for Ipld {
    fn from(read: Generic<P, A>) -> Self {
        read.into()
    }
}

impl<P: TryFrom<Ipld>, A: TryFrom<Ipld>> TryFrom<Ipld> for Generic<P, A> {
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        if let Ipld::Map(mut map) = ipld {
            if map.len() > 2 {
                return Err(()); // FIXME
            }

            Ok(Generic {
                path: map
                    .remove("path")
                    .map(|ipld| P::try_from(ipld).map_err(|_| ()))
                    .transpose()?,

                args: map
                    .remove("args")
                    .map(|ipld| A::try_from(ipld).map_err(|_| ()))
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
    type Parents = crud::Any;
    type ParentError = (); // FIXME

    fn check_parent(&self, other: &crud::Any) -> Result<(), Self::ParentError> {
        if let Some(self_path) = &self.path {
            // FIXME check the args, too!
            if let Some(proof_path) = &other.path {
                if self_path != proof_path {
                    return Err(());
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

        // FIXME
        // if let Some(args_res) = promised.args {
        //     let v = args_res.map(|a| {
        //         // FIXME extract
        //         a.iter().try_fold(BTreeMap::new(), |mut acc, (k, v)| {
        //             acc.insert(*k, (*v).try_into().ok()?);
        //             Some(acc)
        //         })
        //     });

        //     // match v {
        //     //
        //     // }
        //     // named.insert(
        //     //     "args".to_string(),
        //     // );
        // }

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
