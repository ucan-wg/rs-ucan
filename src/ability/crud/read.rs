//! The ability to read (fetch) from a resource.
//!
//! * This ability may be invoked when [`Ready`].
//! * See the [`Builder`] to view the [delegation chain](./type.Builder.html#delegation-hierarchy).
//! * The invocation [Lifecycle](./struct.Ready.html#lifecycle) can be found on [`Ready`] or [`Promised`].

use super::error::ProofError;
use crate::{
    ability::{arguments, command::Command},
    invocation::{promise, promise::Resolves, Resolvable},
    proof::{
        checkable::Checkable, error::OptionalFieldError, parentful::Parentful,
        parents::CheckParents, same::CheckSame, util::check_optional,
    },
};
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg_attr(doc, aquamarine::aquamarine)]
/// The CRUD ability to retrieve data from a resource.
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

    /// Additional arugments to pass in the request.
    pub args: arguments::Named<Ipld>,
}

#[cfg_attr(doc, aquamarine::aquamarine)]
/// The CRUD ability to retrieve data from a resource.
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Builder {
    // FIXME ^^^^^^ rename delegation as a pattern
    /// Optional path within the resource.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,

    #[serde(default, skip_serializing_if = "Option::is_none")]
    /// Additional arugments to pass in the request.
    pub args: Option<arguments::Named<Ipld>>,
}

impl Command for Ready {
    const COMMAND: &'static str = "crud/read";
}

impl Checkable for Builder {
    type Hierarchy = Parentful<Builder>;
}

impl CheckSame for Builder {
    type Error = ProofError;

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        check_optional(self.path.as_ref(), proof.path.as_ref())
            .map_err(Into::<ProofError>::into)?;

        let args = self.args.as_ref().ok_or(ProofError::MissingProofArgs)?;
        if let Some(proof_args) = &proof.args {
            args.contains(proof_args).map_err(Into::into)
        } else {
            Ok(())
        }
    }
}

impl CheckParents for Builder {
    type Parents = super::Any;
    type ParentError = OptionalFieldError;

    fn check_parent(&self, crud_any: &super::Any) -> Result<(), Self::ParentError> {
        if let Some(path) = &self.path {
            let crud_any_path = crud_any.path.as_ref().ok_or(OptionalFieldError::Missing)?;
            if path != crud_any_path {
                return Err(OptionalFieldError::Unequal);
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

    pub args: arguments::Promised,
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

impl From<Ready> for Promised {
    fn from(r: Ready) -> Promised {
        Promised {
            path: promise::PromiseOk::Fulfilled(r.path).into(),
            args: promise::PromiseOk::Fulfilled(r.args.into()).into(),
        }
    }
}

impl From<Promised> for arguments::Named<Ipld> {
    fn from(p: Promised) -> arguments::Named<Ipld> {
        p.into()
    }
}

impl Resolvable for Ready {
    type Promised = Promised;

    fn try_resolve(p: Promised) -> Result<Ready, Promised> {
        match Resolves::try_resolve_2(p.path, p.args) {
            Ok((path, promise_args)) => match promise_args.try_into() {
                Ok(args) => Ok(Ready { path, args }),
                Err(args) => Err(Promised {
                    args: promise::PromiseOk::Fulfilled(args).into(),
                    path: promise::PromiseOk::Fulfilled(path).into(),
                }),
            },
            Err((path, args)) => Err(Promised { path, args }),
        }
    }
}
