//! A plugin for handling the `wnfs` scheme.

use std::fmt::Display;

use crate::semantics::{ability::Ability, caveat::EmptyCaveat, resource::Resource};
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_scheme() {
        assert_eq!(WnfsPlugin.scheme(), "wnfs");
    }

    #[test]
    fn test_plugin_try_handle_resource_public() -> anyhow::Result<()> {
        let resource =
            WnfsPlugin.try_handle_resource(&Url::parse("wnfs://user/public/path/to/file")?)?;

        assert_eq!(
            resource,
            Some(WnfsResource::PublicPath {
                user: "user".to_string(),
                path: vec!["path".to_string(), "to".to_string(), "file".to_string()],
            })
        );

        Ok(())
    }

    #[test]
    fn test_plugin_try_handle_resource_invalid() -> anyhow::Result<()> {
        let resource =
            WnfsPlugin.try_handle_resource(&Url::parse("wnfs://user/invalid/path/to/file")?)?;

        assert_eq!(resource, None);

        Ok(())
    }

    #[test]
    fn test_plugin_try_handle_ability_public() -> anyhow::Result<()> {
        let resource = WnfsResource::PublicPath {
            user: "user".to_string(),
            path: vec!["path".to_string(), "to".to_string(), "file".to_string()],
        };

        let ability_create = WnfsPlugin.try_handle_ability(&resource, "wnfs/create")?;
        let ability_revise = WnfsPlugin.try_handle_ability(&resource, "wnfs/revise")?;
        let ability_soft_delete = WnfsPlugin.try_handle_ability(&resource, "wnfs/soft_delete")?;
        let ability_overwrite = WnfsPlugin.try_handle_ability(&resource, "wnfs/overwrite")?;
        let ability_super_user = WnfsPlugin.try_handle_ability(&resource, "wnfs/super_user")?;
        let ability_invalid = WnfsPlugin.try_handle_ability(&resource, "wnfs/not-an-ability")?;

        assert_eq!(ability_create, Some(WnfsAbility::Create));
        assert_eq!(ability_revise, Some(WnfsAbility::Revise));
        assert_eq!(ability_soft_delete, Some(WnfsAbility::SoftDelete));
        assert_eq!(ability_overwrite, Some(WnfsAbility::Overwrite));
        assert_eq!(ability_super_user, Some(WnfsAbility::SuperUser));
        assert_eq!(ability_invalid, None);

        Ok(())
    }

    #[test]
    fn test_resource_public_display() {
        let resource = WnfsResource::PublicPath {
            user: "user".to_string(),
            path: vec!["foo".to_string(), "bar".to_string()],
        };

        assert_eq!(resource.to_string(), "wnfs://user/public/foo/bar");
    }

    #[test]
    fn test_resource_public_attenuation_identity() {
        let resource = WnfsResource::PublicPath {
            user: "user".to_string(),
            path: vec!["foo".to_string(), "bar".to_string()],
        };

        assert!(resource.is_valid_attenuation(&resource));
    }

    #[test]
    fn test_resource_public_attenuation_child() {
        let parent = WnfsResource::PublicPath {
            user: "user".to_string(),
            path: vec!["foo".to_string(), "bar".to_string()],
        };

        let child = WnfsResource::PublicPath {
            user: "user".to_string(),
            path: vec!["foo".to_string(), "bar".to_string(), "baz".to_string()],
        };

        assert!(child.is_valid_attenuation(&parent));
    }

    #[test]
    fn test_resource_public_attenuation_descendent() {
        let ancestor = WnfsResource::PublicPath {
            user: "user".to_string(),
            path: vec!["foo".to_string(), "bar".to_string()],
        };

        let descendent = WnfsResource::PublicPath {
            user: "user".to_string(),
            path: vec![
                "foo".to_string(),
                "bar".to_string(),
                "baz".to_string(),
                "qux".to_string(),
            ],
        };

        assert!(descendent.is_valid_attenuation(&ancestor));
    }

    #[test]
    fn test_resource_public_attenuation_parent() {
        let parent = WnfsResource::PublicPath {
            user: "user".to_string(),
            path: vec!["foo".to_string(), "bar".to_string()],
        };

        let child = WnfsResource::PublicPath {
            user: "user".to_string(),
            path: vec!["foo".to_string(), "bar".to_string(), "baz".to_string()],
        };

        assert!(!parent.is_valid_attenuation(&child));
    }

    #[test]
    fn test_resource_public_attenuation_ancestor() {
        let ancestor = WnfsResource::PublicPath {
            user: "user".to_string(),
            path: vec!["foo".to_string(), "bar".to_string()],
        };

        let descendent = WnfsResource::PublicPath {
            user: "user".to_string(),
            path: vec![
                "foo".to_string(),
                "bar".to_string(),
                "baz".to_string(),
                "qux".to_string(),
            ],
        };

        assert!(!ancestor.is_valid_attenuation(&descendent));
    }

    #[test]
    fn test_resource_public_attenuation_sibling() {
        let sibling_1 = WnfsResource::PublicPath {
            user: "user".to_string(),
            path: vec!["foo".to_string(), "bar".to_string()],
        };

        let sibling_2 = WnfsResource::PublicPath {
            user: "user".to_string(),
            path: vec!["foo".to_string(), "baz".to_string()],
        };

        assert!(!sibling_1.is_valid_attenuation(&sibling_2));
        assert!(!sibling_2.is_valid_attenuation(&sibling_1));
    }

    #[test]
    fn test_resource_public_attenuation_distinct_users() {
        let path_1 = WnfsResource::PublicPath {
            user: "user1".to_string(),
            path: vec!["foo".to_string(), "bar".to_string()],
        };

        let path_2 = WnfsResource::PublicPath {
            user: "user2".to_string(),
            path: vec!["foo".to_string(), "bar".to_string()],
        };

        assert!(!path_1.is_valid_attenuation(&path_2));
        assert!(!path_2.is_valid_attenuation(&path_1));
    }

    #[test]
    fn test_ability_attenuation() {
        assert!(WnfsAbility::Create.is_valid_attenuation(&WnfsAbility::Create));
        assert!(WnfsAbility::Create.is_valid_attenuation(&WnfsAbility::Revise));
        assert!(WnfsAbility::Create.is_valid_attenuation(&WnfsAbility::SoftDelete));
        assert!(WnfsAbility::Create.is_valid_attenuation(&WnfsAbility::Overwrite));
        assert!(WnfsAbility::Create.is_valid_attenuation(&WnfsAbility::SuperUser));

        assert!(!WnfsAbility::Revise.is_valid_attenuation(&WnfsAbility::Create));
        assert!(WnfsAbility::Revise.is_valid_attenuation(&WnfsAbility::Revise));
        assert!(WnfsAbility::Revise.is_valid_attenuation(&WnfsAbility::SoftDelete));
        assert!(WnfsAbility::Revise.is_valid_attenuation(&WnfsAbility::Overwrite));
        assert!(WnfsAbility::Revise.is_valid_attenuation(&WnfsAbility::SuperUser));

        assert!(!WnfsAbility::SoftDelete.is_valid_attenuation(&WnfsAbility::Create));
        assert!(!WnfsAbility::SoftDelete.is_valid_attenuation(&WnfsAbility::Revise));
        assert!(WnfsAbility::SoftDelete.is_valid_attenuation(&WnfsAbility::SoftDelete));
        assert!(WnfsAbility::SoftDelete.is_valid_attenuation(&WnfsAbility::Overwrite));
        assert!(WnfsAbility::SoftDelete.is_valid_attenuation(&WnfsAbility::SuperUser));

        assert!(!WnfsAbility::Overwrite.is_valid_attenuation(&WnfsAbility::Create));
        assert!(!WnfsAbility::Overwrite.is_valid_attenuation(&WnfsAbility::Revise));
        assert!(!WnfsAbility::Overwrite.is_valid_attenuation(&WnfsAbility::SoftDelete));
        assert!(WnfsAbility::Overwrite.is_valid_attenuation(&WnfsAbility::Overwrite));
        assert!(WnfsAbility::Overwrite.is_valid_attenuation(&WnfsAbility::SuperUser));

        assert!(!WnfsAbility::SuperUser.is_valid_attenuation(&WnfsAbility::Create));
        assert!(!WnfsAbility::SuperUser.is_valid_attenuation(&WnfsAbility::Revise));
        assert!(!WnfsAbility::SuperUser.is_valid_attenuation(&WnfsAbility::SoftDelete));
        assert!(!WnfsAbility::SuperUser.is_valid_attenuation(&WnfsAbility::Overwrite));
        assert!(WnfsAbility::Overwrite.is_valid_attenuation(&WnfsAbility::SuperUser));
    }

    #[test]
    fn test_ability_display() {
        assert_eq!(WnfsAbility::Create.to_string(), "wnfs/create");
        assert_eq!(WnfsAbility::Revise.to_string(), "wnfs/revise");
        assert_eq!(WnfsAbility::SoftDelete.to_string(), "wnfs/soft_delete");
        assert_eq!(WnfsAbility::Overwrite.to_string(), "wnfs/overwrite");
        assert_eq!(WnfsAbility::SuperUser.to_string(), "wnfs/super_user");
    }
}
