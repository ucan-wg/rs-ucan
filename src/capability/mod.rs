use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use url::Url;

use crate::Ucan;

pub mod proof;

#[derive(Serialize, Deserialize)]
pub struct RawCapability {
    with: String,
    can: String,
}

impl<S, A> From<Capability<S, A>> for RawCapability
where
    S: Scope,
    A: Action,
{
    fn from(capability: Capability<S, A>) -> Self {
        RawCapability {
            with: capability.with.to_string(),
            can: capability.can.to_string(),
        }
    }
}

pub trait Scope: ToString + TryFrom<Url> + Clone {
    fn contains(&self, other: &Self) -> bool;
}

pub trait Action: Ord + TryFrom<String> + ToString + Clone {}

#[derive(Clone)]
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

#[derive(Clone)]
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
            With::As { did, kind } => format!("as:{}:{}", did, kind.to_string()),
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

        match path_parts.nth(0) {
            Some("did") => (),
            _ => return None,
        };

        match path_parts.nth(0) {
            Some("key") => (),
            _ => return None,
        };

        let value = match path_parts.nth(0) {
            Some(value) => value,
            _ => return None,
        };

        Some((format!("did:key:{}", value), path_parts.collect()))
    }

    fn parse_resource(&self, with: &Url) -> Option<Resource<S>> {
        Some(match with.path() {
            "*" => Resource::Unscoped,
            _ => Resource::Scoped(self.parse_scope(with)?),
        })
    }

    fn parse(&self, with: String, can: String) -> Option<Capability<S, A>> {
        let uri = Url::parse(with.as_str()).ok()?;

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

        let action = match self.parse_action(&can) {
            Some(action) => action,
            None => return None,
        };

        Some(Capability::new(resource, action))
    }
}

#[derive(Clone)]
pub struct Capability<S, A>
where
    S: Scope,
    A: Action,
{
    with: With<S>,
    can: A,
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

// impl<S, A> Into<RawCapability> for Capability<S, A>
// where
//     S: Scope,
//     A: Action,
// {
//     fn into(self) -> RawCapability {
//         RawCapability {
//             with: self.with.to_string(),
//             can: self.can.to_string(),
//         }
//     }
// }

pub struct CapabilityIterator<'a, Semantics, S, A>
where
    Semantics: CapabilitySemantics<S, A>,
    S: Scope,
    A: Action,
{
    index: usize,
    ucan: &'a Ucan,
    semantics: &'a Semantics,
    capability_type: PhantomData<Capability<S, A>>,
}

impl<'a, Semantics, S, A> CapabilityIterator<'a, Semantics, S, A>
where
    Semantics: CapabilitySemantics<S, A>,
    S: Scope,
    A: Action,
{
    pub fn new(ucan: &'a Ucan, semantics: &'a Semantics) -> Self {
        CapabilityIterator {
            index: 0,
            ucan,
            semantics,
            capability_type: PhantomData::<Capability<S, A>>,
        }
    }
}

impl<'a, Semantics, S, A> Iterator for CapabilityIterator<'a, Semantics, S, A>
where
    Semantics: CapabilitySemantics<S, A>,
    S: Scope,
    A: Action,
{
    type Item = Capability<S, A>;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(capability_json) = self.ucan.attenuation().get(self.index) {
            self.index = self.index + 1;

            let (raw_with, raw_can) = match serde_json::from_value(capability_json.clone()) {
                Ok(RawCapability { with, can }) => (with, can),
                _ => continue,
            };

            match self.semantics.parse(raw_with, raw_can) {
                Some(capability) => return Some(capability),
                None => continue,
            };
        }

        None
    }
}
