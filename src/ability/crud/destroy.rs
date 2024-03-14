//! Destroy a resource.

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
///     destroyready("crud::destroy::Destroy")
///
///     top --> any --> mutate --> destroy
///     destroy -.->|invoke| destroypromise -.->|resolve| destroyready -.-> exe{{execute}}
///
///     style destroyready stroke:orange;
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Destroy {
    /// An optional path to a sub-resource that is to be destroyed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
}

impl From<Destroy> for Ipld {
    fn from(destroy: Destroy) -> Self {
        let mut map = BTreeMap::new();

        if let Some(path) = destroy.path {
            map.insert("path".to_string(), path.display().to_string().into());
        }

        Ipld::Map(map)
    }
}

const COMMAND: &'static str = "/crud/destroy";

impl Command for Destroy {
    const COMMAND: &'static str = COMMAND;
}

impl From<Destroy> for arguments::Named<Ipld> {
    fn from(ready: Destroy) -> Self {
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

impl TryFrom<arguments::Named<Ipld>> for Destroy {
    type Error = TryFromArgsError;

    fn try_from(args: arguments::Named<Ipld>) -> Result<Self, Self::Error> {
        let mut path = None;

        for (k, ipld) in args {
            match k.as_str() {
                "path" => {
                    if let Ipld::String(s) = ipld {
                        path = Some(PathBuf::from(s));
                    } else {
                        return Err(TryFromArgsError::NotAPathBuf);
                    }
                }
                s => return Err(TryFromArgsError::InvalidField(s.into())),
            }
        }

        Ok(Destroy { path })
    }
}

#[derive(Error, Debug, PartialEq, Clone, Serialize, Deserialize)]
pub enum TryFromArgsError {
    #[error("Path value is not a PathBuf")]
    NotAPathBuf,

    #[error("Invalid map key {0}")]
    InvalidField(String),
}

impl promise::Resolvable for Destroy {
    type Promised = PromisedDestroy;
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
///     destroyready("crud::destroy::Destroy")
///
///     top --> any --> mutate --> destroy
///     destroy -.->|invoke| destroypromise -.->|resolve| destroyready -.-> exe{{execute}}
///
///     style destroypromise stroke:orange;
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PromisedDestroy {
    /// An optional path to a sub-resource that is to be destroyed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<promise::Any<PathBuf>>,
}

impl TryFrom<arguments::Named<ipld::Promised>> for PromisedDestroy {
    type Error = FromPromisedArgsError;

    fn try_from(arguments: arguments::Named<ipld::Promised>) -> Result<Self, Self::Error> {
        let mut path = None;

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
                _ => return Err(FromPromisedArgsError::InvalidMapKey(k)),
            }
        }

        Ok(PromisedDestroy { path })
    }
}

#[derive(Error, Debug, PartialEq, Clone)]
pub enum FromPromisedArgsError {
    #[error("Invalid path {0}")]
    InvalidPath(String),

    #[error("Invalid map key {0}")]
    InvalidMapKey(String),
}

impl Command for PromisedDestroy {
    const COMMAND: &'static str = COMMAND;
}

// impl From<PromisedDestroy> for arguments::Named<Ipld> {
//     fn from(promised: PromisedDestroy) -> Self {
//         let mut named = arguments::Named::new();
//
//         if let Some(path_res) = promised.path {
//             named.insert(
//                 "path".to_string(),
//                 path_res.map(|p| ipld::Newtype::from(p).0).into(),
//             );
//         }
//
//         named
//     }
// }

impl From<Destroy> for PromisedDestroy {
    fn from(r: Destroy) -> PromisedDestroy {
        PromisedDestroy {
            path: r
                .path
                .map(|inner_path| promise::Any::Resolved(inner_path).into()),
        }
    }
}

impl From<PromisedDestroy> for arguments::Named<ipld::Promised> {
    fn from(promised: PromisedDestroy) -> Self {
        let mut named = arguments::Named::new();

        if let Some(path) = promised.path {
            named.insert("path".to_string(), path.to_promised_ipld());
        }

        named
    }
}
