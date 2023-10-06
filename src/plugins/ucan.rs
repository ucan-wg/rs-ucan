//! A plugin for handling the `ucan` scheme.

use std::fmt::Display;

use cid::Cid;
use url::Url;

use crate::{
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
        // TODO: I'm not handling the OwnedBy or OwnedByWithScheme cases yet,
        // because the spec probably needs to be modified to treat the DID as
        // a literal, by wrapping it in square brackets, to avoid parsing issues
        // from treating it as an authority with a port.
        match resource_uri.path() {
            "*" => Ok(Some(UcanResource::AllProvable)),
            "./*" => Ok(Some(UcanResource::LocallyProvable)),
            path => {
                if let Ok(cid) = Cid::try_from(path) {
                    return Ok(Some(UcanResource::ByCid(cid)));
                }

                Ok(None)
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
            UcanResource::OwnedBy(did) => format!("//{}/*", did),
            UcanResource::OwnedByWithScheme(did, scheme) => format!("//{}/{}", did, scheme),
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
        write!(f, "ucan/*")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_scheme() {
        assert_eq!(UcanPlugin.scheme(), "ucan");
    }

    #[test]
    fn test_plugin_try_handle_resource_by_cid() -> anyhow::Result<()> {
        let resource = UcanPlugin.try_handle_resource(&Url::parse(
            "ucan:bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
        )?)?;

        assert_eq!(
            resource,
            Some(UcanResource::ByCid(Cid::try_from(
                "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi"
            )?))
        );

        Ok(())
    }

    #[test]
    fn test_plugin_try_handle_resource_all_provable() -> anyhow::Result<()> {
        let resource = UcanPlugin.try_handle_resource(&Url::parse("ucan:*")?)?;

        assert_eq!(resource, Some(UcanResource::AllProvable));

        Ok(())
    }

    #[test]
    fn test_plugin_try_handle_resource_locally_provable() -> anyhow::Result<()> {
        let resource = UcanPlugin.try_handle_resource(&Url::parse("ucan:./*")?)?;

        assert_eq!(resource, Some(UcanResource::LocallyProvable));

        Ok(())
    }

    #[test]
    #[ignore = "Spec expects DID not to be URL encoded, but this results in invalid URLs"]
    fn test_plugin_try_handle_resource_owned_by() -> anyhow::Result<()> {
        let resource = UcanPlugin.try_handle_resource(&Url::parse(
            "ucan://did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK/*",
        )?)?;

        assert_eq!(
            resource,
            Some(UcanResource::OwnedBy(
                "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string()
            ))
        );

        Ok(())
    }

    #[test]
    #[ignore = "Spec expects DID not to be URL encoded, but this results in invalid URLs"]
    fn test_plugin_try_handle_resource_owned_with_scheme() -> anyhow::Result<()> {
        let resource = UcanPlugin.try_handle_resource(&Url::parse(
            "ucan://did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK/wnfs",
        )?)?;

        assert_eq!(
            resource,
            Some(UcanResource::OwnedByWithScheme(
                "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
                "wnfs".to_string()
            ))
        );

        Ok(())
    }

    #[test]
    fn test_plugin_try_handle_ability_delegation() -> anyhow::Result<()> {
        let ability = UcanPlugin.try_handle_ability(&UcanResource::AllProvable, "ucan/*")?;

        assert_eq!(ability, Some(UcanAbilityDelegation));

        Ok(())
    }

    #[test]
    fn test_resource_by_cid_display() -> anyhow::Result<()> {
        let resource = UcanResource::ByCid(Cid::try_from(
            "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
        )?);

        assert_eq!(
            resource.to_string(),
            "ucan:bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi"
        );

        Ok(())
    }

    #[test]
    fn test_resource_all_provable_display() {
        let resource = UcanResource::AllProvable;

        assert_eq!(resource.to_string(), "ucan:*");
    }

    #[test]
    fn test_resource_locally_provable_display() {
        let resource = UcanResource::LocallyProvable;

        assert_eq!(resource.to_string(), "ucan:./*");
    }

    #[test]
    fn test_resource_owned_by_display() {
        let resource = UcanResource::OwnedBy(
            "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        );

        assert_eq!(
            resource.to_string(),
            "ucan://did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK/*"
        );
    }

    #[test]
    fn test_resource_owned_by_with_scheme_display() {
        let resource = UcanResource::OwnedByWithScheme(
            "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
            "wnfs".to_string(),
        );

        assert_eq!(
            resource.to_string(),
            "ucan://did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK/wnfs"
        );
    }

    #[test]
    fn test_ability_delegation_display() {
        let ability = UcanAbilityDelegation;

        assert_eq!(ability.to_string(), "ucan/*");
    }

    #[test]
    fn test_resource_attenuation() -> anyhow::Result<()> {
        let all_provable = UcanResource::AllProvable;
        let locally_provable = UcanResource::LocallyProvable;

        let by_cid_1 = UcanResource::ByCid(Cid::try_from(
            "bafybeigdyrzt5sfp7udm7hu76uh7y26nf3efuylqabf3oclgtqy55fbzdi",
        )?);

        let by_cid_2 = UcanResource::ByCid(Cid::try_from(
            "QmbWqxBEKC3P8tqsKc98xmWNzrzDtRLMiMPL8wBuTGsMnR",
        )?);

        let owned_by_1 = UcanResource::OwnedBy(
            "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
        );

        let owned_by_2 = UcanResource::OwnedBy("did:example:123456789abcdefghi".to_string());

        let owned_by_with_scheme_1 = UcanResource::OwnedByWithScheme(
            "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
            "wnfs".to_string(),
        );

        let owned_by_with_scheme_2 = UcanResource::OwnedByWithScheme(
            "did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK".to_string(),
            "ucan".to_string(),
        );

        assert!(all_provable.is_valid_attenuation(&all_provable));
        assert!(!all_provable.is_valid_attenuation(&locally_provable));
        assert!(!all_provable.is_valid_attenuation(&by_cid_1));
        assert!(!all_provable.is_valid_attenuation(&by_cid_2));
        assert!(!all_provable.is_valid_attenuation(&owned_by_1));
        assert!(!all_provable.is_valid_attenuation(&owned_by_2));
        assert!(!all_provable.is_valid_attenuation(&owned_by_with_scheme_1));
        assert!(!all_provable.is_valid_attenuation(&owned_by_with_scheme_2));

        assert!(!locally_provable.is_valid_attenuation(&all_provable));
        assert!(locally_provable.is_valid_attenuation(&locally_provable));
        assert!(!locally_provable.is_valid_attenuation(&by_cid_1));
        assert!(!locally_provable.is_valid_attenuation(&by_cid_2));
        assert!(!locally_provable.is_valid_attenuation(&owned_by_1));
        assert!(!locally_provable.is_valid_attenuation(&owned_by_2));
        assert!(!locally_provable.is_valid_attenuation(&owned_by_with_scheme_1));
        assert!(!locally_provable.is_valid_attenuation(&owned_by_with_scheme_2));

        assert!(!by_cid_1.is_valid_attenuation(&all_provable));
        assert!(!by_cid_1.is_valid_attenuation(&locally_provable));
        assert!(by_cid_1.is_valid_attenuation(&by_cid_1));
        assert!(!by_cid_1.is_valid_attenuation(&by_cid_2));
        assert!(!by_cid_1.is_valid_attenuation(&owned_by_1));
        assert!(!by_cid_1.is_valid_attenuation(&owned_by_2));
        assert!(!by_cid_1.is_valid_attenuation(&owned_by_with_scheme_1));
        assert!(!by_cid_1.is_valid_attenuation(&owned_by_with_scheme_2));

        assert!(!by_cid_2.is_valid_attenuation(&all_provable));
        assert!(!by_cid_2.is_valid_attenuation(&locally_provable));
        assert!(!by_cid_2.is_valid_attenuation(&by_cid_1));
        assert!(by_cid_2.is_valid_attenuation(&by_cid_2));
        assert!(!by_cid_2.is_valid_attenuation(&owned_by_1));
        assert!(!by_cid_2.is_valid_attenuation(&owned_by_2));
        assert!(!by_cid_2.is_valid_attenuation(&owned_by_with_scheme_1));
        assert!(!by_cid_2.is_valid_attenuation(&owned_by_with_scheme_2));

        assert!(!owned_by_1.is_valid_attenuation(&all_provable));
        assert!(!owned_by_1.is_valid_attenuation(&locally_provable));
        assert!(!owned_by_1.is_valid_attenuation(&by_cid_1));
        assert!(!owned_by_1.is_valid_attenuation(&by_cid_2));
        assert!(owned_by_1.is_valid_attenuation(&owned_by_1));
        assert!(!owned_by_1.is_valid_attenuation(&owned_by_2));
        assert!(!owned_by_1.is_valid_attenuation(&owned_by_with_scheme_1));
        assert!(!owned_by_1.is_valid_attenuation(&owned_by_with_scheme_2));

        assert!(!owned_by_2.is_valid_attenuation(&all_provable));
        assert!(!owned_by_2.is_valid_attenuation(&locally_provable));
        assert!(!owned_by_2.is_valid_attenuation(&by_cid_1));
        assert!(!owned_by_2.is_valid_attenuation(&by_cid_2));
        assert!(!owned_by_2.is_valid_attenuation(&owned_by_1));
        assert!(owned_by_2.is_valid_attenuation(&owned_by_2));
        assert!(!owned_by_2.is_valid_attenuation(&owned_by_with_scheme_1));
        assert!(!owned_by_2.is_valid_attenuation(&owned_by_with_scheme_2));

        assert!(!owned_by_with_scheme_1.is_valid_attenuation(&all_provable));
        assert!(!owned_by_with_scheme_1.is_valid_attenuation(&locally_provable));
        assert!(!owned_by_with_scheme_1.is_valid_attenuation(&by_cid_1));
        assert!(!owned_by_with_scheme_1.is_valid_attenuation(&by_cid_2));
        assert!(!owned_by_with_scheme_1.is_valid_attenuation(&owned_by_1));
        assert!(!owned_by_with_scheme_1.is_valid_attenuation(&owned_by_2));
        assert!(owned_by_with_scheme_1.is_valid_attenuation(&owned_by_with_scheme_1));
        assert!(!owned_by_with_scheme_1.is_valid_attenuation(&owned_by_with_scheme_2));

        assert!(!owned_by_with_scheme_2.is_valid_attenuation(&all_provable));
        assert!(!owned_by_with_scheme_2.is_valid_attenuation(&locally_provable));
        assert!(!owned_by_with_scheme_2.is_valid_attenuation(&by_cid_1));
        assert!(!owned_by_with_scheme_2.is_valid_attenuation(&by_cid_2));
        assert!(!owned_by_with_scheme_2.is_valid_attenuation(&owned_by_1));
        assert!(!owned_by_with_scheme_2.is_valid_attenuation(&owned_by_2));
        assert!(!owned_by_with_scheme_2.is_valid_attenuation(&owned_by_with_scheme_1));
        assert!(owned_by_with_scheme_2.is_valid_attenuation(&owned_by_with_scheme_2));

        Ok(())
    }

    #[test]
    fn test_ability_attenuation() -> anyhow::Result<()> {
        let ability = UcanAbilityDelegation;

        assert!(ability.is_valid_attenuation(&ability));

        Ok(())
    }
}
