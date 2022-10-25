use crate::serde::ser_to_lower_case;
use anyhow::anyhow;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::fmt::Debug;
use url::Url;

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct CapabilityIpld {
    pub with: String,
    #[serde(serialize_with = "ser_to_lower_case")]
    pub can: String,
    pub nb: Option<Value>,
}

impl<S: Scope, A: Action> From<&Capability<S, A>> for CapabilityIpld {
    fn from(capability: &Capability<S, A>) -> Self {
        CapabilityIpld {
            with: capability.with().to_string(),
            can: capability.can().to_string(),

            // TODO(#22): Full support for 0.9 and the nb field
            nb: None,
        }
    }
}

impl TryFrom<&Value> for CapabilityIpld {
    type Error = anyhow::Error;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value {
            Value::Object(map) => {
                let with = map
                    .get("with")
                    .ok_or_else(|| anyhow!("Missing 'with' field"))?;
                let can = map
                    .get("can")
                    .ok_or_else(|| anyhow!("Missing 'can' field"))?;
                let nb = map.get("nb").cloned();

                let with = match with {
                    Value::String(with) => with.clone(),
                    _ => return Err(anyhow!("The 'with' field must be a string")),
                };

                let can = match can {
                    Value::String(can) => can.to_lowercase(),
                    _ => return Err(anyhow!("The 'can' field must be a string")),
                };

                Ok(CapabilityIpld { with, can, nb })
            }
            _ => Err(anyhow!("Not a valid capability: {}", value)),
        }
    }
}

pub trait Scope: ToString + TryFrom<Url> + PartialEq + Clone {
    fn contains(&self, other: &Self) -> bool;
}

pub trait Action: Ord + TryFrom<String> + ToString + Clone {}

#[derive(Clone, Eq, PartialEq)]
pub enum Resource<S>
where
    S: Scope,
{
    Scoped(S),
    Unscoped,
}

impl<S> Resource<S>
where
    S: Scope,
{
    pub fn contains(&self, other: &Self) -> bool {
        match self {
            Resource::Unscoped => true,
            Resource::Scoped(scope) => match other {
                Resource::Scoped(other_scope) => scope.contains(other_scope),
                _ => false,
            },
        }
    }
}

impl<S> ToString for Resource<S>
where
    S: Scope,
{
    fn to_string(&self) -> String {
        match self {
            Resource::Unscoped => "*".into(),
            Resource::Scoped(value) => value.to_string(),
        }
    }
}

#[derive(Clone, Eq, PartialEq)]
pub enum With<S>
where
    S: Scope,
{
    Resource { kind: Resource<S> },
    My { kind: Resource<S> },
    As { did: String, kind: Resource<S> },
}

impl<S> With<S>
where
    S: Scope,
{
    pub fn contains(&self, other: &Self) -> bool {
        match (self, other) {
            (
                With::Resource { kind: resource },
                With::Resource {
                    kind: other_resource,
                },
            ) => resource.contains(other_resource),
            (
                With::My { kind: resource },
                With::My {
                    kind: other_resource,
                },
            ) => resource.contains(other_resource),
            (
                With::As {
                    did,
                    kind: resource,
                },
                With::As {
                    did: other_did,
                    kind: other_resource,
                },
            ) if did == other_did => resource.contains(other_resource),
            _ => false,
        }
    }
}

impl<S> ToString for With<S>
where
    S: Scope,
{
    fn to_string(&self) -> String {
        match self {
            With::Resource { kind } => kind.to_string(),
            With::My { kind } => format!("my:{}", kind.to_string()),
            With::As { did, kind } => format!("as:{did}:{}", kind.to_string()),
        }
    }
}

pub trait CapabilitySemantics<S, A>
where
    S: Scope,
    A: Action,
{
    fn parse_scope(&self, scope: &Url) -> Option<S> {
        S::try_from(scope.clone()).ok()
    }
    fn parse_action(&self, can: &str) -> Option<A> {
        A::try_from(String::from(can)).ok()
    }

    fn extract_did(&self, path: &str) -> Option<(String, String)> {
        let mut path_parts = path.split(':');

        match path_parts.next() {
            Some("did") => (),
            _ => return None,
        };

        match path_parts.next() {
            Some("key") => (),
            _ => return None,
        };

        let value = match path_parts.next() {
            Some(value) => value,
            _ => return None,
        };

        Some((format!("did:key:{value}"), path_parts.collect()))
    }

    fn parse_resource(&self, with: &Url) -> Option<Resource<S>> {
        Some(match with.path() {
            "*" => Resource::Unscoped,
            _ => Resource::Scoped(self.parse_scope(with)?),
        })
    }

    fn parse(&self, with: &str, can: &str) -> Option<Capability<S, A>> {
        let uri = Url::parse(with).ok()?;

        let resource = match uri.scheme() {
            "my" => With::My {
                kind: self.parse_resource(&uri)?,
            },
            "as" => {
                let (did, with) = self.extract_did(uri.path())?;
                let with = Url::parse(with.as_str()).ok()?;

                With::As {
                    did,
                    kind: self.parse_resource(&with)?,
                }
            }
            _ => With::Resource {
                kind: self.parse_resource(&uri)?,
            },
        };

        let action = match self.parse_action(can) {
            Some(action) => action,
            None => return None,
        };

        Some(Capability::new(resource, action))
    }
}

#[derive(Clone, Eq, PartialEq)]
pub struct Capability<S, A>
where
    S: Scope,
    A: Action,
{
    pub with: With<S>,
    pub can: A,
}

impl<S, A> Debug for Capability<S, A>
where
    S: Scope,
    A: Action,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Capability")
            .field("with", &self.with.to_string())
            .field("can", &self.can.to_string())
            .finish()
    }
}

impl<S, A> Capability<S, A>
where
    S: Scope,
    A: Action,
{
    pub fn new(with: With<S>, can: A) -> Self {
        Capability { with, can }
    }

    pub fn enables(&self, other: &Capability<S, A>) -> bool {
        self.with.contains(&other.with) && self.can >= other.can
    }

    pub fn with(&self) -> &With<S> {
        &self.with
    }

    pub fn can(&self) -> &A {
        &self.can
    }
}
