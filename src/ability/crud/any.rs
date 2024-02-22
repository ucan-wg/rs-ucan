//! "Any" CRUD ability (superclass of all CRUD abilities)

use crate::{
    ability::{arguments, command::Command},
    proof::{error::OptionalFieldError, parentless::NoParents, same::CheckSame},
};
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[cfg_attr(doc, aquamarine::aquamarine)]
/// The superclass of all other CRUD abilities.
///
/// For example, the [`crud::Create`][super::create::Create] ability may
/// be proven by the [`crud::Any`][Any] ability in a delegation chain.
///
/// It may not be invoked directly, but rather is used as a delegaton proof
/// for other CRUD abilities (see the diagram below).
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
///       mutate("crud/mutate")
///
///       subgraph Invokable
///         read("crud/read")
///         create("crud/create")
///         update("crud/update")
///         destroy("crud/destroy")
///       end
///     end
///
///     readrun{{"invoke"}}
///     createrun{{"invoke"}}
///     updaterun{{"invoke"}}
///     destroyrun{{"invoke"}}
///
///     top --> any
///             any --> read -.-> readrun
///             any --> mutate
///                     mutate --> create -.-> createrun
///                     mutate --> update -.-> updaterun
///                     mutate --> destroy -.-> destroyrun
///
///     style any stroke:orange;
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Any {
    /// A an optional path relative to the actor's root.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,
}

impl Command for Any {
    const COMMAND: &'static str = "crud/*";
}

impl TryFrom<arguments::Named<Ipld>> for Any {
    type Error = ();

    fn try_from(arguments: arguments::Named<Ipld>) -> Result<Self, Self::Error> {
        let mut path = None;

        for (key, value) in arguments.iter() {
            match key.as_str() {
                "path" => {
                    let some_path = match value {
                        Ipld::String(s) => Ok(PathBuf::from(s)),
                        _ => Err(()),
                    }?;

                    path = Some(some_path);
                }
                _ => return Err(()),
            }
        }

        Ok(Any { path })
    }
}

// FIXME pipe example

impl NoParents for Any {}

impl CheckSame for Any {
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

impl TryFrom<Ipld> for Any {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl From<Any> for Ipld {
    fn from(builder: Any) -> Self {
        builder.into()
    }
}

impl From<Any> for arguments::Named<Ipld> {
    fn from(any: Any) -> arguments::Named<Ipld> {
        let mut named = arguments::Named::new();
        if let Some(path) = any.path {
            named.insert(
                "path".into(),
                path.into_os_string()
                    .into_string()
                    .expect("PathBuf should generate a valid path")
                    .into(),
            );
        }
        named
    }
}
