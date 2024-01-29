use crate::{
    ability::traits::Command,
    proof::{checkable::Checkable, parentful::Parentful, parents::CheckParents, same::CheckSame},
};
use libipld_core::{ipld::Ipld, serde as ipld_serde};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use url::Url;

use super::parents::Mutable;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Create {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uri: Option<Url>,

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
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld).map_err(|_| ())
    }
}

impl Checkable for Create {
    type CheckAs = Parentful<Create>;
}

impl CheckSame for Create {
    type Error = (); // FIXME better error
    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        if self.uri == proof.uri {
            Ok(())
        } else {
            Err(())
        }
    }
}

impl CheckParents for Create {
    type Parents = Mutable;
    type ParentError = ();

    fn check_parents(&self, other: &Self::Parents) -> Result<(), Self::ParentError> {
        if let Some(self_uri) = &self.uri {
            match other {
                Mutable::Any(any) => {
                    if let Some(proof_uri) = &any.uri {
                        if self_uri != proof_uri {
                            return Err(());
                        }
                    }
                }
                Mutable::Mutate(mutate) => {
                    if let Some(proof_uri) = &mutate.uri {
                        if self_uri != proof_uri {
                            return Err(());
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
