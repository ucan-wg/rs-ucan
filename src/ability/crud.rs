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

pub mod create;
pub mod destroy;
pub mod error;
pub mod parents;
pub mod read;
pub mod update;

pub use any::Any;
pub use mutate::Mutate;

#[cfg(target_arch = "wasm32")]
pub mod js;
