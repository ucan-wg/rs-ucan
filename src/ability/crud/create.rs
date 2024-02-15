//! Create new resources.
use super::{error::ProofError, parents::MutableParents};
use crate::{
    ability::{arguments, command::Command},
    delegation::Delegable,
    invocation::{promise, promise::Resolves, Resolvable},
    ipld,
    proof::{
        checkable::Checkable, error::OptionalFieldError, parentful::Parentful,
        parents::CheckParents, same::CheckSame, util::check_optional,
    },
};
use libipld_core::{error::SerdeError, ipld::Ipld, serde as ipld_serde};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{collections::BTreeMap, path::PathBuf};

// FIXME deserialize instance

/// A helper for creating lifecycle instances of `crud/create` with the correct shape.
#[derive(Debug, Clone, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Generic<Path, Args> {
    /// An optional path to a sub-resource that is to be created.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<Path>,

    /// Optional arguments for creation.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub args: Option<Args>,
}

#[cfg_attr(doc, aquamarine::aquamarine)]
/// The executable/dispatchable variant of the `crud/create` ability.
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
///           create("crud/create")
///         end
///       end
///     end
///
///     createpromise("crud::create::Promised")
///     createready("crud::create::Ready")
///
///     top --> any --> mutate --> create
///     create -.->|invoke| createpromise -.->|resolve| createready -.-> exe{{execute}}
///
///     style createready stroke:orange;
/// ```
pub type Ready = Generic<PathBuf, arguments::Named<Ipld>>;

#[cfg_attr(doc, aquamarine::aquamarine)]
/// The delegatable ability for creating other agents.
///
/// # Lifecycle
///
/// The lifecycle of a `crud/create` ability is as follows:
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
///           create("crud/create")
///         end
///       end
///     end
///
///     createpromise("crud::create::Promised")
///     createready("crud::create::Ready")
///
///     top --> any --> mutate --> create
///     create -.->|invoke| createpromise -.->|resolve| createready -.-> exe{{execute}}
///
///     style create stroke:orange;
/// ```
pub type Builder = Generic<PathBuf, arguments::Named<Ipld>>;

#[cfg_attr(doc, aquamarine::aquamarine)]
/// An invoked `crud/create` ability (but possibly awaiting another
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
///           create("crud/create")
///         end
///       end
///     end
///
///     createpromise("crud::create::Promised")
///     createready("crud::create::Ready")
///
///     top --> any --> mutate --> create
///     create -.->|invoke| createpromise -.->|resolve| createready -.-> exe{{execute}}
///
///     style createpromise stroke:orange;
/// ```
pub type Promised = Generic<promise::Resolves<PathBuf>, arguments::Promised>;

impl<P, A> Command for Generic<P, A> {
    const COMMAND: &'static str = "crud/create";
}

impl<P: Into<Ipld>, A: Into<Ipld>> From<Generic<P, A>> for Ipld {
    fn from(create: Generic<P, A>) -> Self {
        create.into()
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

impl Delegable for Ready {
    type Builder = Builder;
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
    type Parents = MutableParents;
    type ParentError = (); // FIXME

    fn check_parent(&self, other: &Self::Parents) -> Result<(), Self::ParentError> {
        if let Some(self_path) = &self.path {
            match other {
                MutableParents::Any(any) => {
                    // FIXME check the args, too!
                    if let Some(proof_path) = &any.path {
                        if self_path != proof_path {
                            return Err(());
                        }
                    }
                }
                MutableParents::Mutate(mutate) => {
                    // FIXME check the args, too!
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

impl From<Promised> for arguments::Named<Ipld> {
    fn from(promised: Promised) -> Self {
        let mut named = arguments::Named::new();

        if let Some(path_res) = promised.path {
            named.insert(
                "path".to_string(),
                path_res.map(|p| ipld::Newtype::from(p).0).into(),
            );
        }

        if let Some(args_res) = promised.args {
            named.insert(
                "args".to_string(),
                args_res
                    .map(|a| {
                        // FIXME extract
                        a.iter()
                            .map(|(k, v)| (k.to_string(), v.clone().serialize_as_ipld()))
                            .collect::<BTreeMap<String, Ipld>>()
                    })
                    .into(),
            );
        }

        named
    }
}

// impl From<arguments::Named<Ipld>> for Promised {
//     fn from(source: arguments::Named<Ipld>) -> Self {
//         let path = source
//             .get("path")
//             .map(|ipld| ipld.clone().try_into().unwrap());
//
//         let args = source
//             .get("args")
//             .map(|ipld| ipld.clone().try_into().unwrap());
//         Promised { path, args }
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

// impl From<Promised> for Builder {
//     fn from(promised: Promised) -> Self {
//         Builder {
//             path: promised.path.map(Into::into),
//             args: promised.args.map(Into::into),
//         }
//     }
// }

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
