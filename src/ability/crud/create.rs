use crate::{
    ability::command::Command,
    proof::{checkable::Checkable, parentful::Parentful, parents::CheckParents, same::CheckSame},
};
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

use super::parents::MutableParents;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Create {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<PathBuf>,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub args: Option<BTreeMap<String, Ipld>>,
}

impl Command for Create {
    const COMMAND: &'static str = "crud/create";
}

impl From<Create> for Ipld {
    fn from(create: Create) -> Self {
        create.into()
    }
}

impl TryFrom<Ipld> for Create {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl Checkable for Create {
    type Hierarchy = Parentful<Create>;
}

impl CheckSame for Create {
    type Error = (); // FIXME better error

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        if self.path == proof.path {
            Ok(())
        } else {
            Err(())
        }
    }
}

impl CheckParents for Create {
    type Parents = MutableParents;
    type ParentError = ();

    fn check_parent(&self, other: &Self::Parents) -> Result<(), Self::ParentError> {
        if let Some(self_path) = &self.path {
            match other {
                MutableParents::Any(any) => {
                    if let Some(proof_path) = &any.path {
                        if self_path != proof_path {
                            return Err(());
                        }
                    }
                }
                MutableParents::Mutate(mutate) => {
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
