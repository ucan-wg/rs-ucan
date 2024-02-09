//! The delegation superclass for all mutable CRUD actions.

use crate::{
    ability::command::Command,
    proof::{
        checkable::Checkable, error::OptionalFieldError, parentful::Parentful,
        parents::CheckParents, same::CheckSame,
    },
};
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[cfg_attr(doc, aquamarine::aquamarine)]
/// The delegation superclass for all mutable CRUD actions.
///
/// For example, the [`crud::Create`][super::create::Create] ability may
/// be proven by the [`crud::Mutate`][Mutate] ability in a delegation chain.
/// [`crud::Any`][super::Any] is a suitable proof for [`crud::Mutate`][Mutate], but not vice-versa.
///
/// It may not be invoked directly, but rather is used as a delegaton proof
/// for other CRUD abilities (see the diagram below).
///
/// # Delegation Hierarchy
///
/// The hierarchy of mutable CRUD abilities is as follows:
///
/// ```mermaid
/// flowchart TB
///     top("*")
///
///     subgraph Message Abilities
///       any("crud/*")
///
///       mutate("crud/mutate")
///
///       subgraph Invokable
///         create("crud/create")
///         update("crud/update")
///         destroy("crud/destroy")
///       end
///     end
///
///     createrun{{"invoke"}}
///     updaterun{{"invoke"}}
///     destroyrun{{"invoke"}}
///
///     top --> any
///             any --> mutate
///                     mutate --> create -.-> createrun
///                     mutate --> update -.-> updaterun
///                     mutate --> destroy -.-> destroyrun
///
///     style mutate stroke:orange;
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Mutate {
    /// A an optional path relative to the actor's root.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
}

impl Command for Mutate {
    const COMMAND: &'static str = "crud/mutate";
}

impl From<Mutate> for Ipld {
    fn from(mutate: Mutate) -> Self {
        mutate.into()
    }
}

impl TryFrom<Ipld> for Mutate {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl Checkable for Mutate {
    type Hierarchy = Parentful<Mutate>;
}

impl CheckSame for Mutate {
    type Error = OptionalFieldError;

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        if let Some(path) = &self.path {
            let proof_path = proof.path.as_ref().ok_or(OptionalFieldError::Missing)?;
            if path != proof_path {
                return Err(OptionalFieldError::Unequal);
            }
        }

        Ok(())
    }
}

impl CheckParents for Mutate {
    type Parents = super::Any;
    type ParentError = OptionalFieldError;

    fn check_parent(&self, crud_any: &Self::Parents) -> Result<(), Self::ParentError> {
        if let Some(path) = &self.path {
            let proof_path = crud_any.path.as_ref().ok_or(OptionalFieldError::Missing)?;
            if path != proof_path {
                return Err(OptionalFieldError::Unequal);
            }
        }

        Ok(())
    }
}
