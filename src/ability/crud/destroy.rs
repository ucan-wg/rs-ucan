//! Destroy a resource.
use super::parents::MutableParents;
use crate::{
    ability::{arguments, command::Command},
    delegation::Delegable,
    invocation::promise,
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
pub struct Generic<Path> {
    /// An optional path to a sub-resource that is to be destroyed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<Path>,
}

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
///     destroyready("crud::destroy::Ready")
///
///     top --> any --> mutate --> destroy
///     destroy -.->|invoke| destroypromise -.->|resolve| destroyready -.-> exe{{execute}}
///
///     style destroyready stroke:orange;
/// ```
pub type Ready = Generic<PathBuf>;

#[cfg_attr(doc, aquamarine::aquamarine)]
/// The delegatable ability for destroying resources.
///
/// # Lifecycle
///
/// The lifecycle of a `crud/destroy` ability is as follows:
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
///     destroyready("crud::destroy::Ready")
///
///     top --> any --> mutate --> destroy
///     destroy -.->|invoke| destroypromise -.->|resolve| destroyready -.-> exe{{execute}}
///
///     style destroy stroke:orange;
/// ```
pub type Builder = Generic<PathBuf>;

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
///     destroyready("crud::destroy::Ready")
///
///     top --> any --> mutate --> destroy
///     destroy -.->|invoke| destroypromise -.->|resolve| destroyready -.-> exe{{execute}}
///
///     style destroypromise stroke:orange;
/// ```
pub type Promised = Generic<promise::Resolves<PathBuf>>;

impl<P> Command for Generic<P> {
    const COMMAND: &'static str = "crud/destroy";
}

impl Delegable for Ready {
    type Builder = Builder;
}

impl<P: Into<Ipld>> From<Generic<P>> for Ipld {
    fn from(destroy: Generic<P>) -> Self {
        destroy.into()
    }
}

impl<P: TryFrom<Ipld>> TryFrom<Ipld> for Generic<P> {
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        if let Ipld::Map(mut map) = ipld {
            if map.len() > 1 {
                return Err(()); // FIXME
            }

            Ok(Generic {
                path: map
                    .remove("path")
                    .map(|ipld| P::try_from(ipld).map_err(|_| ()))
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
    type Parents = MutableParents;
    type ParentError = (); // FIXME

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

impl From<Promised> for arguments::Named<Ipld> {
    fn from(promised: Promised) -> Self {
        let mut named = arguments::Named::new();

        if let Some(path_res) = promised.path {
            named.insert(
                "path".to_string(),
                path_res.map(|p| ipld::Newtype::from(p).0).into(),
            );
        }

        named
    }
}

impl From<Ready> for Promised {
    fn from(r: Ready) -> Promised {
        Promised {
            path: r
                .path
                .map(|inner_path| promise::PromiseOk::Fulfilled(inner_path).into()),
        }
    }
}

impl promise::Resolvable for Ready {
    type Promised = Promised;

    fn try_resolve(p: Promised) -> Result<Ready, Promised> {
        // FIXME extract & cleanup
        let path = match p.path {
            Some(ref res_path) => match res_path.clone().try_resolve() {
                Ok(path) => Some(Ok(path)),
                Err(unresolved) => Some(Err(Promised {
                    path: Some(unresolved),
                })),
            },
            None => None,
        }
        .transpose()?;

        Ok(Ready { path })
    }
}

impl From<Promised> for Ready {
    fn from(p: Promised) -> Ready {
        Ready {
            path: p
                .path
                .map(|inner_path| inner_path.try_resolve().ok())
                .flatten(),
        }
    }
}
