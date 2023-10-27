//! Capabilities, and traits for deserializing them

use std::collections::BTreeMap;

use serde::{
    de::{DeserializeSeed, IgnoredAny, Visitor},
    Deserialize, Deserializer, Serialize, Serializer,
};
use url::Url;

use crate::semantics::{
    ability::{Ability, TopAbility},
    caveat::{Caveat, EmptyCaveat},
    resource::Resource,
};

/// The default capability handler, when deserializing a UCAN
pub type DefaultCapabilityParser = PluginCapability;

/// A capability
#[derive(Debug, Clone)]
pub struct Capability {
    /// The resource
    resource: Box<dyn Resource>,
    /// The ability
    ability: Box<dyn Ability>,
    /// The caveat
    caveat: Box<dyn Caveat>,
}

impl Capability {
    /// Creates a new capability
    pub fn new<R, A, C>(resource: R, ability: A, caveat: C) -> Self
    where
        R: Resource,
        A: Ability,
        C: Caveat,
    {
        Self {
            resource: Box::new(resource),
            ability: Box::new(ability),
            caveat: Box::new(caveat),
        }
    }

    /// Creates a new capability by cloning the resource, ability, and caveat as trait objects
    pub fn clone_box(resource: &dyn Resource, ability: &dyn Ability, caveat: &dyn Caveat) -> Self {
        Self {
            resource: dyn_clone::clone_box(resource),
            ability: dyn_clone::clone_box(ability),
            caveat: dyn_clone::clone_box(caveat),
        }
    }

    /// Returns the resource
    pub fn resource(&self) -> &dyn Resource {
        &*self.resource
    }

    /// Returns the ability
    pub fn ability(&self) -> &dyn Ability {
        &*self.ability
    }

    /// Returns the caveat
    pub fn caveat(&self) -> &dyn Caveat {
        &*self.caveat
    }

    /// Returns true if self is subsumed by other
    pub fn is_subsumed_by(&self, other: &Capability) -> bool {
        if !self.resource.is_valid_attenuation(&*other.resource) {
            return false;
        }

        if !(other.ability.is::<TopAbility>() || self.ability.is_valid_attenuation(&*other.ability))
        {
            return false;
        }

        other.caveat.is::<EmptyCaveat>() || self.caveat.is_valid_attenuation(&*other.caveat)
    }
}

/// A collection of capabilities
#[derive(Clone, Debug)]
pub struct Capabilities<C> {
    inner: Vec<Capability>,
    _marker: std::marker::PhantomData<fn() -> C>,
}

impl<C> Default for Capabilities<C> {
    fn default() -> Self {
        Self {
            inner: Default::default(),
            _marker: Default::default(),
        }
    }
}

impl<C> Capabilities<C> {
    /// Creates a new collection of capabilities from a vector
    pub fn new(inner: Vec<Capability>) -> Self {
        Self {
            inner,
            _marker: Default::default(),
        }
    }

    /// Pushes a capability to the collection
    pub fn push(&mut self, capability: Capability) {
        self.inner.push(capability);
    }

    /// Extends the collection with the capabilities from a slice of capabilities
    pub fn extend_from_slice(&mut self, capabilities: &[Capability]) {
        self.inner.extend_from_slice(capabilities);
    }

    /// Returns an iterator over the capabilities
    pub fn iter(&self) -> impl Iterator<Item = &Capability> {
        self.inner.iter()
    }
}

/// Handles deserializing capabilities
pub trait CapabilityParser: Clone {
    /// Tries to deserialize a capability from a resource_uri, ability, and a deserilizer for the caveat
    fn try_handle(
        resource_uri: &Url,
        ability: &str,
        caveat_deserializer: &mut dyn erased_serde::Deserializer<'_>,
    ) -> Result<Option<Capability>, anyhow::Error>
    where
        Self: Sized;
}

/// A capability handler that deserializes using the registered plugins
#[derive(Clone, Debug)]
pub struct PluginCapability {}

impl CapabilityParser for PluginCapability {
    fn try_handle(
        resource_uri: &Url,
        ability: &str,
        caveat_deserializer: &mut dyn erased_serde::Deserializer<'_>,
    ) -> Result<Option<Capability>, anyhow::Error> {
        let resource_scheme = resource_uri.scheme();

        for plugin in crate::plugins::plugins().filter(|p| p.scheme() == resource_scheme) {
            let Some(resource) = plugin.try_handle_resource(resource_uri)? else {
                continue;
            };

            let Some(ability) = plugin.try_handle_ability(&resource, ability)? else {
                continue;
            };

            let Some(caveat) = plugin.try_handle_caveat(&resource, &ability, caveat_deserializer)?
            else {
                continue;
            };

            return Ok(Some(Capability {
                resource,
                ability,
                caveat,
            }));
        }

        Ok(None)
    }
}

impl<Cap> Serialize for Capabilities<Cap>
where
    Cap: CapabilityParser,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut capabilities: BTreeMap<String, BTreeMap<String, Vec<&dyn Caveat>>> =
            Default::default();

        for capability in self.iter() {
            let resource_uri = capability.resource().to_string();
            let ability_key = capability.ability().to_string();
            let caveat = capability.caveat();

            capabilities
                .entry(resource_uri)
                .or_default()
                .entry(ability_key)
                .or_default()
                .push(caveat);
        }

        capabilities.serialize(serializer)
    }
}

impl<'de, C> Deserialize<'de> for Capabilities<C>
where
    C: CapabilityParser,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CapabilitiesVisitor<C> {
            _marker: std::marker::PhantomData<fn() -> C>,
        }

        impl<'de, C> Visitor<'de> for CapabilitiesVisitor<C>
        where
            C: CapabilityParser,
        {
            type Value = Vec<Capability>;

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "a map of capabilities")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                let mut capabilities = Vec::new();

                while let Some(resource_key) = map.next_key::<String>()? {
                    let resource_uri =
                        Url::parse(&resource_key).map_err(serde::de::Error::custom)?;

                    map.next_value_seed(Abilities::<C> {
                        resource_uri,
                        capabilities: &mut capabilities,
                        _marker: Default::default(),
                    })?;
                }

                Ok(capabilities)
            }
        }

        let caps = deserializer.deserialize_map(CapabilitiesVisitor::<C> {
            _marker: Default::default(),
        })?;

        Ok(Self::new(caps))
    }
}

struct Abilities<'a, C> {
    resource_uri: Url,
    capabilities: &'a mut Vec<Capability>,
    _marker: std::marker::PhantomData<fn() -> C>,
}

impl<'de, 'a, C> DeserializeSeed<'de> for Abilities<'a, C>
where
    C: CapabilityParser,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct AbilitiesVisitor<'a, C> {
            resource_uri: Url,
            capabilities: &'a mut Vec<Capability>,
            _marker: std::marker::PhantomData<fn() -> C>,
        }

        impl<'de, 'a, C> Visitor<'de> for AbilitiesVisitor<'a, C>
        where
            C: CapabilityParser,
        {
            type Value = ();

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(formatter, "a map of abilities for {}", self.resource_uri)
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::MapAccess<'de>,
            {
                while let Some(ability_key) = map.next_key::<String>()? {
                    map.next_value_seed(Caveats::<C> {
                        resource_uri: self.resource_uri.clone(),
                        ability_key: ability_key.clone(),
                        capabilities: self.capabilities,
                        _marker: Default::default(),
                    })?;
                }

                Ok(())
            }
        }

        deserializer.deserialize_map(AbilitiesVisitor::<C> {
            resource_uri: self.resource_uri,
            capabilities: self.capabilities,
            _marker: Default::default(),
        })
    }
}

struct Caveats<'a, C> {
    resource_uri: Url,
    ability_key: String,
    capabilities: &'a mut Vec<Capability>,
    _marker: std::marker::PhantomData<fn() -> C>,
}

impl<'de, 'a, C> DeserializeSeed<'de> for Caveats<'a, C>
where
    C: CapabilityParser,
{
    type Value = ();

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct CaveatsVisitor<'a, C> {
            resource_uri: Url,
            ability_key: String,
            capabilities: &'a mut Vec<Capability>,
            _marker: std::marker::PhantomData<fn() -> C>,
        }

        impl<'de, 'a, C> Visitor<'de> for CaveatsVisitor<'a, C>
        where
            C: CapabilityParser,
        {
            type Value = ();

            fn expecting(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(
                    formatter,
                    "a map of caveats for {} : {}",
                    self.resource_uri, self.ability_key
                )
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: serde::de::SeqAccess<'de>,
            {
                while let Some(element) = seq.next_element_seed(CaveatSeed::<C> {
                    resource_uri: self.resource_uri.clone(),
                    ability_key: self.ability_key.clone(),
                    _marker: Default::default(),
                })? {
                    if let Some(capability) = element {
                        self.capabilities.push(capability);
                    }
                }

                Ok(())
            }
        }

        deserializer.deserialize_seq(CaveatsVisitor::<C> {
            resource_uri: self.resource_uri,
            ability_key: self.ability_key,
            capabilities: self.capabilities,
            _marker: Default::default(),
        })
    }
}

struct CaveatSeed<Cap> {
    resource_uri: Url,
    ability_key: String,
    _marker: std::marker::PhantomData<fn() -> Cap>,
}

impl<'de, Cap> DeserializeSeed<'de> for CaveatSeed<Cap>
where
    Cap: CapabilityParser,
{
    type Value = Option<Capability>;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut deserializer = <dyn erased_serde::Deserializer<'_>>::erase(deserializer);

        let Some(capability) =
            Cap::try_handle(&self.resource_uri, &self.ability_key, &mut deserializer)
                .map_err(serde::de::Error::custom)?
        else {
            erased_serde::deserialize::<IgnoredAny>(&mut deserializer)
                .map_err(serde::de::Error::custom)?;
            return Ok(None);
        };

        Ok(Some(capability))
    }
}
