//! A plugin for handling the `wnfs` scheme.

use std::fmt::Display;

use crate::{
    error::Error,
    semantics::{ability::Ability, caveat::EmptyCaveat, resource::Resource},
};
use url::Url;

use super::Plugin;

/// A plugin for handling the `wnfs` scheme.
#[derive(Debug)]
pub struct WnfsPlugin;

crate::register_plugin!(WNFS, &WnfsPlugin);

impl Plugin for WnfsPlugin {
    type Resource = WnfsResource;
    type Ability = WnfsAbility;
    type Caveat = EmptyCaveat;

    type Error = anyhow::Error;

    fn scheme(&self) -> &'static str {
        "wnfs"
    }

    fn try_handle_resource(
        &self,
        resource_uri: &Url,
    ) -> Result<Option<Self::Resource>, Self::Error> {
        let Some(user) = resource_uri.host_str() else {
            return Ok(None);
        };

        let Some(path_segments) = resource_uri.path_segments() else {
            return Ok(None);
        };

        match path_segments.collect::<Vec<_>>().as_slice() {
            ["public", path @ ..] => Ok(Some(WnfsResource::PublicPath {
                user: user.to_string(),
                path: path.iter().map(|s| s.to_string()).collect(),
            })),
            ["private", ..] => todo!(),
            _ => Ok(None),
        }
    }

    fn try_handle_ability(
        &self,
        _resource: &Self::Resource,
        ability: &str,
    ) -> Result<Option<Self::Ability>, Self::Error> {
        match ability {
            "wnfs/create" => Ok(Some(WnfsAbility::Create)),
            "wnfs/revise" => Ok(Some(WnfsAbility::Revise)),
            "wnfs/soft_delete" => Ok(Some(WnfsAbility::SoftDelete)),
            "wnfs/overwrite" => Ok(Some(WnfsAbility::Overwrite)),
            "wnfs/super_user" => Ok(Some(WnfsAbility::SuperUser)),
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

/// A resource for the `wnfs` scheme.
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum WnfsResource {
    /// wnfs://<user>/public/<path>
    PublicPath {
        /// The user
        user: String,
        /// The path
        path: Vec<String>,
    },
    /// wnfs://<user>/private/<acc>
    PrivatePath {
        /// The user
        user: String,
    }, // TODO
}

impl Resource for WnfsResource {
    fn is_valid_attenuation(&self, other: &dyn Resource) -> bool {
        let Some(other) = other.downcast_ref::<WnfsResource>() else {
            return false;
        };

        match self {
            WnfsResource::PublicPath { user, path } => {
                let WnfsResource::PublicPath {
                    user: other_user,
                    path: other_path,
                } = other
                else {
                    return false;
                };

                if user != other_user {
                    return false;
                }

                path.strip_prefix(other_path.as_slice()).is_some()
            }
            WnfsResource::PrivatePath { .. } => todo!(),
        }
    }
}

impl Display for WnfsResource {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WnfsResource::PublicPath { user, path } => {
                f.write_fmt(format_args!("wnfs://{}/public/{}", user, path.join("/")))
            }

            WnfsResource::PrivatePath { .. } => todo!(),
        }
    }
}

/// An ability for the `wnfs` scheme.
#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum WnfsAbility {
    /// wnfs/create
    Create,
    /// wnfs/revise
    Revise,
    /// wnfs/soft_delete
    SoftDelete,
    /// wnfs/overwrite
    Overwrite,
    /// wnfs/super_user
    SuperUser,
}

impl Ability for WnfsAbility {
    fn is_valid_attenuation(&self, other: &dyn Ability) -> bool {
        let Some(other) = other.downcast_ref::<WnfsAbility>() else {
            return false;
        };

        self <= other
    }
}

impl Display for WnfsAbility {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WnfsAbility::Create => f.write_str("wnfs/create"),
            WnfsAbility::Revise => f.write_str("wnfs/revise"),
            WnfsAbility::SoftDelete => f.write_str("wnfs/soft_delete"),
            WnfsAbility::Overwrite => f.write_str("wnfs/overwrite"),
            WnfsAbility::SuperUser => f.write_str("wnfs/super_user"),
        }
    }
}
