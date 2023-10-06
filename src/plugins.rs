//! Plugins for definining custom semantics

use core::fmt;
use std::sync::RwLock;

use downcast_rs::{impl_downcast, Downcast};
use linkme::distributed_slice;
use url::Url;

use crate::{
    error::Error,
    semantics::{
        ability::{Ability, TopAbility},
        caveat::{Caveat, EmptyCaveat},
        resource::Resource,
    },
};

pub mod ucan;
pub mod wnfs;

#[distributed_slice]
#[doc(hidden)]
pub static STATIC_PLUGINS: [&dyn Plugin<
    Resource = Box<dyn Resource>,
    Ability = Box<dyn Ability>,
    Caveat = Box<dyn Caveat>,
    Error = Error,
>] = [..];

type ErasedPlugin = dyn Plugin<
    Resource = Box<dyn Resource>,
    Ability = Box<dyn Ability>,
    Caveat = Box<dyn Caveat>,
    Error = Error,
>;

lazy_static::lazy_static! {
    static ref RUNTIME_PLUGINS: RwLock<Vec<&'static ErasedPlugin>> = RwLock::new(Vec::new());
}

/// A plugin for handling a specific scheme
pub trait Plugin: Send + Sync + Downcast + 'static {
    /// The type of resource this plugin handles
    type Resource;

    /// The type of ability this plugin handles
    type Ability;

    /// The type of caveat this plugin handles
    type Caveat;

    /// The type of error this plugin may return
    type Error;

    /// The scheme this plugin handles
    fn scheme(&self) -> &'static str;

    /// Handle a resource
    fn try_handle_resource(
        &self,
        resource_uri: &Url,
    ) -> Result<Option<Self::Resource>, Self::Error>;

    /// Handle an ability
    fn try_handle_ability(
        &self,
        resource: &Self::Resource,
        ability: &str,
    ) -> Result<Option<Self::Ability>, Self::Error>;

    /// Handle a caveat
    fn try_handle_caveat(
        &self,
        resource: &Self::Resource,
        ability: &Self::Ability,
        deserializer: &mut dyn erased_serde::Deserializer<'_>,
    ) -> Result<Option<Self::Caveat>, Self::Error>;
}

impl_downcast!(Plugin assoc Resource, Ability, Caveat, Error);

/// A wrapped plugin that unifies plugin error handling, and handles common semantics, such
/// as top abilities.
pub struct WrappedPlugin<R, A, C, E>
where
    R: 'static,
    A: 'static,
    C: 'static,
    E: 'static,
{
    #[doc(hidden)]
    pub inner: &'static dyn Plugin<Resource = R, Ability = A, Caveat = C, Error = E>,
}

impl<R, A, C, E> Plugin for WrappedPlugin<R, A, C, E>
where
    R: Resource,
    A: Ability,
    C: Caveat,
    E: Into<anyhow::Error>,
{
    type Resource = Box<dyn Resource>;
    type Ability = Box<dyn Ability>;
    type Caveat = Box<dyn Caveat>;

    type Error = Error;

    fn scheme(&self) -> &'static str {
        self.inner.scheme()
    }

    fn try_handle_resource(
        &self,
        resource_uri: &Url,
    ) -> Result<Option<Self::Resource>, Self::Error> {
        self.inner.try_handle_resource(resource_uri).map_or_else(
            |e| Err(Error::PluginError(anyhow::anyhow!(e).into())),
            |r| Ok(r.map(|r| Box::new(r) as Box<dyn Resource>)),
        )
    }

    fn try_handle_ability(
        &self,
        resource: &Self::Resource,
        ability: &str,
    ) -> Result<Option<Box<dyn Ability>>, Self::Error> {
        if ability == "*" {
            return Ok(Some(Box::new(TopAbility)));
        }

        let Some(resource) = resource.downcast_ref::<R>() else {
            return Ok(None);
        };

        self.inner
            .try_handle_ability(resource, ability)
            .map_or_else(
                |e| Err(Error::PluginError(anyhow::anyhow!(e).into())),
                |a| Ok(a.map(|a| Box::new(a) as Box<dyn Ability>)),
            )
    }

    fn try_handle_caveat(
        &self,
        resource: &Self::Resource,
        ability: &Self::Ability,
        deserializer: &mut dyn erased_serde::Deserializer<'_>,
    ) -> Result<Option<Self::Caveat>, Self::Error> {
        let Some(resource) = resource.downcast_ref::<R>() else {
            return Ok(None);
        };

        if ability.is::<TopAbility>() {
            return Ok(Some(Box::new(
                erased_serde::deserialize::<EmptyCaveat>(deserializer)
                    .map_err(|e| anyhow::anyhow!(e))?,
            )));
        }

        let Some(ability) = ability.downcast_ref::<A>() else {
            return Ok(None);
        };

        self.inner
            .try_handle_caveat(resource, ability, deserializer)
            .map_or_else(
                |e| Err(Error::PluginError(anyhow::anyhow!(e).into())),
                |c| Ok(c.map(|c| Box::new(c) as Box<dyn Caveat>)),
            )
    }
}

impl<R, A, C, E> fmt::Debug for WrappedPlugin<R, A, C, E> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("WrappedPlugin")
            .field("scheme", &self.inner.scheme())
            .finish()
    }
}

/// Get an iterator over all plugins
pub fn plugins() -> impl Iterator<
    Item = &'static dyn Plugin<
        Resource = Box<dyn Resource>,
        Ability = Box<dyn Ability>,
        Caveat = Box<dyn Caveat>,
        Error = Error,
    >,
> {
    let static_plugins = STATIC_PLUGINS.iter().copied();
    let runtime_plugins = RUNTIME_PLUGINS
        .read()
        .expect("plugin lock poisoned")
        .clone()
        .into_iter();

    static_plugins.chain(runtime_plugins)
}

/// Register a plugin
pub fn register_plugin<R, A, C, E>(
    plugin: &'static dyn Plugin<Resource = R, Ability = A, Caveat = C, Error = E>,
) where
    R: Resource,
    A: Ability,
    C: Caveat,
    E: Into<anyhow::Error>,
{
    let erased = Box::new(WrappedPlugin { inner: plugin });
    let leaked = Box::leak::<'static>(erased);

    RUNTIME_PLUGINS
        .write()
        .expect("plugin lock poisoned")
        .push(leaked);
}

/// Register a plugin at compile time
#[macro_export]
macro_rules! register_plugin {
    ($name:ident, $plugin:expr) => {
        #[linkme::distributed_slice($crate::plugins::STATIC_PLUGINS)]
        static $name: &'static dyn Plugin<
            Resource = Box<dyn $crate::semantics::resource::Resource>,
            Ability = Box<dyn $crate::semantics::ability::Ability>,
            Caveat = Box<dyn $crate::semantics::caveat::Caveat>,
            Error = $crate::error::Error,
        > = &$crate::plugins::WrappedPlugin { inner: $plugin };
    };
}
