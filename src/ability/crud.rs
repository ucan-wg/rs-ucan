//! Abilties for [CRUD] (create, read, update, and destroy) interfaces
//!
//! An overview of the hierarchy can be found on [`crud::Any`][`Any`].
//!
//! # Wrapping External Resources
//!
//! In most cases, the Subject _is_ the resource being acted
//! on with a CRUD interface. To model external resources directly
//! (i.e. without a URL), generate a unique [`Did`] that represents the
//! specific resource (i.e. the `sub`) directly. This makes the
//! UCAN self-certifying, and can give multiple names to a single
//! resource (which can be important if operating over an open network
//! such as a DHT or gossip). It also provides an abstraction if,
//! for example, the the domain name of a service changes.
//!
//! # `path` Field
//!
//! All variants of CRUD abilities include an *optional* `path` field.
//!
//! There are cases where a Subject acts as a gateway for *external*
//! resources, such as web services or hierarchical file systems.
//! Both of these contain sub-resources expressed via path.
//! If you are issued access to the root, and can attenuate that access to
//! any sub-path, or a single leaf resource.
//!
//! ```js
//! {
//!   "sub: "did:example:1234", // <-- e.g. Wraps a web API
//!   "cmd": "crud/update",
//!   "args": {
//!       "path": "/some/path/to/a/resource",
//!   },
//!   // ...
//! }
//! ```
//!
//! [CRUD]: https://en.wikipedia.org/wiki/Create,_read,_update_and_delete
//! [`Did`]: crate::did::Did

mod any;
mod mutate;
mod parents;

pub mod create;
pub mod destroy;
pub mod error;
pub mod read;
pub mod update;

pub use any::Any;
pub use mutate::Mutate;
pub use parents::MutableParents;

use crate::{ability::arguments, invocation::Resolvable, proof::same::CheckSame};
use libipld_core::ipld::Ipld;

#[cfg(target_arch = "wasm32")]
pub mod js;

#[derive(Debug, Clone, PartialEq)]
pub enum Ready {
    Create(create::Ready),
    Read(read::Ready),
    Update(update::Ready),
    Destroy(destroy::Ready),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Builder {
    Create(create::Builder),
    Read(read::Builder),
    Update(update::Builder),
    Destroy(destroy::Builder),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Promised {
    Create(create::Promised),
    Read(read::Promised),
    Update(update::Promised),
    Destroy(destroy::Promised),
}

impl From<Promised> for arguments::Named<Ipld> {
    fn from(promised: Promised) -> Self {
        match promised {
            Promised::Create(create) => create.into(),
            Promised::Read(read) => read.into(),
            Promised::Update(update) => update.into(),
            Promised::Destroy(destroy) => destroy.into(),
        }
    }
}

impl From<Ready> for Builder {
    fn from(ready: Ready) -> Self {
        match ready {
            Ready::Create(create) => Builder::Create(create.into()),
            Ready::Read(read) => Builder::Read(read.into()),
            Ready::Update(update) => Builder::Update(update.into()),
            Ready::Destroy(destroy) => Builder::Destroy(destroy.into()),
        }
    }
}

impl TryFrom<Builder> for Ready {
    type Error = (); // FIXME

    fn try_from(builder: Builder) -> Result<Self, Self::Error> {
        match builder {
            Builder::Create(create) => create.try_into().map(Ready::Create).map_err(|_| ()),
            Builder::Read(read) => read.try_into().map(Ready::Read).map_err(|_| ()),
            Builder::Update(update) => update.try_into().map(Ready::Update).map_err(|_| ()),
            Builder::Destroy(destroy) => destroy.try_into().map(Ready::Destroy).map_err(|_| ()),
        }
    }
}

impl CheckSame for Builder {
    type Error = ();

    fn check_same(&self, other: &Self) -> Result<(), Self::Error> {
        match (self, other) {
            (Builder::Create(a), Builder::Create(b)) => a.check_same(b),
            (Builder::Read(a), Builder::Read(b)) => a.check_same(b),
            (Builder::Update(a), Builder::Update(b)) => a.check_same(b),
            (Builder::Destroy(a), Builder::Destroy(b)) => a.check_same(b),
            _ => Err(()),
        }
    }
}

impl Resolvable for Ready {
    type Promised = Promised;

    fn try_resolve(promised: Promised) -> Result<Self, Self::Promised> {
        match promised {
            Promised::Create(create) => Resolvable::try_resolve(create)
                .map(Ready::Create)
                .map_err(Promised::Create),
            Promised::Read(read) => Resolvable::try_resolve(read)
                .map(Ready::Read)
                .map_err(Promised::Read),
            Promised::Update(update) => Resolvable::try_resolve(update)
                .map(Ready::Update)
                .map_err(Promised::Update),
            Promised::Destroy(destroy) => Resolvable::try_resolve(destroy)
                .map(Ready::Destroy)
                .map_err(Promised::Destroy),
        }
    }
}
