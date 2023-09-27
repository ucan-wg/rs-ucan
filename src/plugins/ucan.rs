//! A plugin for handling the `ucan` scheme.

use std::fmt::Display;

use cid::Cid;
use url::Url;

use crate::{
    error::Error,
    semantics::{ability::Ability, caveat::EmptyCaveat},
    Did,
};

use super::{Plugin, Resource};

/// A plugin for handling the `ucan` scheme.
#[derive(Debug)]
pub struct UcanPlugin;

crate::register_plugin!(UCAN, &UcanPlugin);

impl Plugin for UcanPlugin {
    type Resource = UcanResource;
    type Ability = UcanAbilityDelegation;
    type Caveat = EmptyCaveat;

    type Error = anyhow::Error;

    fn scheme(&self) -> &'static str {
        "ucan"
    }

    fn try_handle_resource(
        &self,
        resource_uri: &Url,
    ) -> Result<Option<Self::Resource>, Self::Error> {
        match resource_uri.path() {
            "*" => Ok(Some(UcanResource::AllProvable)),
            "./*" => Ok(Some(UcanResource::LocallyProvable)),
            path => {
                if let Ok(cid) = Cid::try_from(path) {
                    return Ok(Some(UcanResource::ByCid(cid)));
                }

                match resource_uri
                    .path_segments()
                    .map(|p| p.collect::<Vec<_>>())
                    .as_deref()
                {
                    Some([did, "*"]) => Ok(Some(UcanResource::OwnedBy(did.to_string()))),
                    Some([did, scheme]) => Ok(Some(UcanResource::OwnedByWithScheme(
                        did.to_string(),
                        scheme.to_string(),
                    ))),
                    _ => Ok(None),
                }
            }
        }
    }

    fn try_handle_ability(
        &self,
        _resource: &Self::Resource,
        ability: &str,
    ) -> Result<Option<Self::Ability>, Self::Error> {
        match ability {
            "ucan/*" => Ok(Some(UcanAbilityDelegation)),
            _ => Ok(None),
        }
    }

    fn try_handle_caveat(
        &self,
        _resource: &Self::Resource,
        _ability: &Self::Ability,
        deserializer: &mut dyn erased_serde::Deserializer<'_>,
    ) -> Result<Option<Self::Caveat>, Self::Error> {
        erased_serde::deserialize(deserializer).map_err(|e| anyhow::anyhow!(e))
    }
}

/// A resource for the `ucan` scheme.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum UcanResource {
    /// ucan:<cid>
    ByCid(Cid),
    /// ucan:*
    AllProvable,
    /// ucan:./*
    LocallyProvable,
    /// ucan://<did>/*
    OwnedBy(Did),
    /// ucan://<did>/<scheme>
    OwnedByWithScheme(Did, String),
}

impl Display for UcanResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let hier_part = match self {
            UcanResource::ByCid(cid) => cid.to_string(),
            UcanResource::AllProvable => "*".to_string(),
            UcanResource::LocallyProvable => "./*".to_string(),
            UcanResource::OwnedBy(did) => format!("{}/*", did),
            UcanResource::OwnedByWithScheme(did, scheme) => format!("{}/{}", did, scheme),
        };

        f.write_fmt(format_args!("ucan:{}", hier_part))
    }
}

impl Resource for UcanResource {
    fn is_valid_attenuation(&self, other: &dyn Resource) -> bool {
        if let Some(resource) = other.downcast_ref::<Self>() {
            return self == resource;
        };

        false
    }
}

/// The UCAN delegation ability from the `ucan` scheme.
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UcanAbilityDelegation;

impl Ability for UcanAbilityDelegation {
    fn is_valid_attenuation(&self, other: &dyn Ability) -> bool {
        if let Some(ability) = other.downcast_ref::<Self>() {
            return self == ability;
        };

        false
    }
}

impl Display for UcanAbilityDelegation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "*")
    }
}
