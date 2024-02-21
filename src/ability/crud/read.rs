//! Read from a resource.

use super::any as crud;
use crate::{
    ability::{
        arguments,
        command::{Command, ParseAbility, ParseAbilityError},
    },
    delegation::Delegable,
    invocation::{promise, promise::Resolves},
    ipld,
    proof::{checkable::Checkable, parentful::Parentful, parents::CheckParents, same::CheckSame},
};
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

// FIXME deserialize instance

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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ready {
    /// An optional path to a sub-resource that is to be read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,

    /// Optional arguments to modify the read request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<arguments::Named<Ipld>>,
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
///     readready("crud::read::Ready")
///
///     top --> any --> read
///     read -.->|invoke| readpromise -.->|resolve| readready -.-> exe{{execute}}
///
///     style readpromise stroke:orange;
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Promised {
    /// An optional path to a sub-resource that is to be read.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<promise::Resolves<PathBuf>>,

    /// Optional arguments to modify the read request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<promise::Resolves<arguments::Named<ipld::Promised>>>,
}

const COMMAND: &'static str = "crud/read";

impl Command for Ready {
    const COMMAND: &'static str = "crud/read";
}

impl Command for Promised {
    const COMMAND: &'static str = "crud/read";
}

impl Delegable for Ready {
    type Builder = Ready;
}

// FIXME resolves vs resolvable is confusing

impl TryFrom<Ipld> for Ready {
    type Error = SerdeError; // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
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

impl TryFrom<arguments::Named<Ipld>> for Ready {
    type Error = ();

    fn try_from(arguments: arguments::Named<Ipld>) -> Result<Ready, Self::Error> {
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

        Ok(Ready { path, args })
    }
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

// impl From<Promised> for arguments::Named<Ipld> {
//     fn from(promised: Promised) -> Self {
//         let mut named = arguments::Named::new();
//
//         if let Some(path_res) = promised.path {
//             named.insert("path".into(), path_res.into());
//         }
//
//         if let Some(args_res) = promised.args {
//             named.insert("args".into(), args_res.into());
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

impl promise::Resolvable for Ready {
    type Promised = Promised;
}

impl From<Promised> for arguments::Named<ipld::Promised> {
    fn from(promised: Promised) -> Self {
        let mut named = arguments::Named::new();

        if let Some(path_res) = promised.path {
            named.insert("path".to_string(), path_res.into());
        }

        if let Some(args_res) = promised.args {
            named.insert("args".to_string(), args_res.into());
        }

        named
    }
}

// impl TryFrom<arguments::Named<Ipld>> for Promised {
//     type Error = ();
//
//     fn try_from(arguments: arguments::Named<Ipld>) -> Result<Promised, Self::Error> {
//         let mut path = None;
//         let mut args = None;
//
//         for (k, v) in arguments.into_iter() {
//             match k.as_str() {
//                 "path" => {
//                     let path = promise::Resolves::try_from(v)?;
//                     path.map(|path| Promised { path, args });
//                 }
//                 "args" => {
//                     let args = promise::Resolves::try_from(v)?;
//                     args.map(|args| Promised { path, args });
//                 }
//                 _ => return Err(()),
//             }
//         }
//
//         Ok(Promised { path, args })
//     }
// }
