//! The ability to read (fetch) from a resource.
//!
//! * This ability may be invoked when [`Ready`].
//! * See the [`Builder`] to view the [delegation chain](./type.Builder.html#delegation-hierarchy).
//! * The invocation [Lifecycle](./struct.Ready.html#lifecycle) can be found on [`Ready`] or [`Promised`].

use super::error::{PathError, ProofError};
use crate::{
    ability::{arguments, command::Command},
    invocation::promise,
    proof::{checkable::Checkable, parentful::Parentful, parents::CheckParents, same::CheckSame},
};
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(doc, aquamarine::aquamarine)]
/// The CRUD ability to retrieve data from a resource.
///
/// Note that the delegation [`Builder`] has the exact same
/// fields in this case.
///
/// # Invocation
///
/// The executable/dispatchable variant of the `msg/send` ability.
///
/// # Lifecycle
///
/// The hierarchy of message abilities is as follows:
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
///     readrun("crud::read::Ready")
///
///     top --> any
///     any --> read -.->|invoke| readpromise -.->|resolve| readrun -.-> exe{{execute}}
///
///     style readrun stroke:orange;
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Ready {
    /// Optional path within the resource.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,

    /// Optional additional arugments to pass in the request.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<arguments::Named<Ipld>>,
}

#[cfg_attr(doc, aquamarine::aquamarine)]
/// The CRUD ability to retrieve data from a resource.
///
/// Note that the delegation [`Builder`] has the exact same
/// fields as [`read::Ready`][Ready] in this case.
///
/// # Delegation Hierarchy
///
/// The hierarchy of CRUD abilities is as follows:
///
/// ```mermaid
/// flowchart TB
///     top("*")
///
///     subgraph Message Abilities
///       any("crud/*")
///
///       subgraph Invokable
///         read("crud/read")
///       end
///     end
///
///     readrun{{"invoke"}}
///
///     top --> any
///             any --> read -.-> readrun
///
///     style read stroke:orange;
/// ```
pub type Builder = Ready;

impl Command for Ready {
    const COMMAND: &'static str = "crud/read";
}

impl Checkable for Ready {
    type Hierarchy = Parentful<Ready>;
}

impl CheckSame for Ready {
    type Error = ProofError;

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        if let Some(path) = &self.path {
            if path != proof.path.as_ref().unwrap() {
                return Err(PathError::Mismatch.into());
            }
        }

        if let Some(args) = &self.args {
            let proof_args = proof.args.as_ref().ok_or(ProofError::MissingProofArgs)?;
            for (k, v) in args.iter() {
                if proof_args
                    .get(k)
                    .ok_or(arguments::NamedError::FieldMissing(k.clone()))?
                    .ne(v)
                {
                    return Err(arguments::NamedError::FieldValueMismatch(k.clone()).into());
                }
            }
        }

        Ok(())
    }
}

impl CheckParents for Ready {
    type Parents = super::Any;
    type ParentError = PathError;

    fn check_parent(&self, crud_any: &super::Any) -> Result<(), Self::ParentError> {
        if let Some(path) = &self.path {
            let crud_any_path = crud_any.path.as_ref().ok_or(PathError::Missing)?;
            if path != crud_any_path {
                return Err(PathError::Mismatch);
            }
        }

        Ok(())
    }
}

impl From<Ready> for Ipld {
    fn from(ready: Ready) -> Self {
        ready.into()
    }
}
impl TryFrom<Ipld> for Ready {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

#[cfg_attr(doc, aquamarine::aquamarine)]
/// This ability is used to fetch messages from other actors.
///
/// # Lifecycle
///
/// The hierarchy of message abilities is as follows:
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
///     readrun("crud::read::Ready")
///
///     top --> any
///     any --> read -.->|invoke| readpromise -.->|resolve| readrun -.-> exe{{execute}}
///
///     style readpromise stroke:orange;
/// ```
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Promised {
    /// Optional path within the resource
    #[serde(skip_serializing_if = "promise::Resolves::resolved_none")]
    pub path: promise::Resolves<Option<PathBuf>>,

    /// Optional additional arugments to pass in the request
    #[serde(skip_serializing_if = "promise::Resolves::resolved_none")]
    pub args: promise::Resolves<Option<arguments::Named<promise::Resolves<Ipld>>>>,
}

impl From<Promised> for Ipld {
    fn from(promised: Promised) -> Self {
        promised.into()
    }
}

impl TryFrom<Ipld> for Promised {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}
