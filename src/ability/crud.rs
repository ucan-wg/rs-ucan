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
//!   "cmd": "/crud/update",
//!   "args": {
//!       "path": "/some/path/to/a/resource",
//!   },
//!   // ...
//! }
//! ```
//!
//! [CRUD]: https://en.wikipedia.org/wiki/Create,_read,_update_and_delete
//! [`Did`]: crate::did::Did

// mod any;
// mod mutate;
// mod parents;

pub mod create;
pub mod destroy;
// pub mod error;
pub mod read;
pub mod update;

// pub use any::Any;
// pub use mutate::Mutate;
// pub use parents::*;

use crate::{
    ability::{
        arguments,
        command::ToCommand,
        parse::{ParseAbility, ParseAbilityError, ParsePromised},
    },
    invocation::promise::Resolvable,
    ipld,
};
use libipld_core::ipld::Ipld;

#[cfg(target_arch = "wasm32")]
pub mod js;

#[derive(Debug, Clone, PartialEq)]
pub enum Ready {
    Create(create::Create),
    Read(read::Ready),
    Update(update::Ready),
    Destroy(destroy::Ready),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Promised {
    Create(create::PromisedCreate),
    Read(read::Promised),
    Update(update::Promised),
    Destroy(destroy::Promised),
}

impl ParsePromised for Promised {
    type PromisedArgsError = ();

    fn try_parse_promised(
        cmd: &str,
        args: arguments::Named<ipld::Promised>,
    ) -> Result<Self, ParseAbilityError<Self::PromisedArgsError>> {
        match create::PromisedCreate::try_parse_promised(cmd, args.clone()) {
            Ok(create) => return Ok(Promised::Create(create)),
            Err(ParseAbilityError::InvalidArgs(_)) => {
                return Err(ParseAbilityError::InvalidArgs(()))
            }
            Err(ParseAbilityError::UnknownCommand(_)) => (),
        }

        match read::Promised::try_parse_promised(cmd, args.clone()) {
            Ok(read) => return Ok(Promised::Read(read)),
            Err(ParseAbilityError::InvalidArgs(_)) => {
                return Err(ParseAbilityError::InvalidArgs(()))
            }
            Err(ParseAbilityError::UnknownCommand(_)) => (),
        }

        match update::Promised::try_parse_promised(cmd, args.clone()) {
            Ok(update) => return Ok(Promised::Update(update)),
            Err(ParseAbilityError::InvalidArgs(_)) => {
                return Err(ParseAbilityError::InvalidArgs(()))
            }
            Err(ParseAbilityError::UnknownCommand(_)) => (),
        }

        match destroy::Promised::try_parse_promised(cmd, args) {
            Ok(destroy) => return Ok(Promised::Destroy(destroy)),
            Err(ParseAbilityError::InvalidArgs(_)) => {
                return Err(ParseAbilityError::InvalidArgs(()))
            }
            Err(ParseAbilityError::UnknownCommand(_)) => (),
        }

        Err(ParseAbilityError::UnknownCommand(cmd.into()))
    }
}

impl ParseAbility for Ready {
    type ArgsErr = ();

    fn try_parse(
        cmd: &str,
        args: arguments::Named<Ipld>,
    ) -> Result<Self, ParseAbilityError<Self::ArgsErr>> {
        match create::Create::try_parse(cmd, args.clone()) {
            Ok(create) => return Ok(Ready::Create(create)),
            Err(ParseAbilityError::InvalidArgs(_)) => {
                return Err(ParseAbilityError::InvalidArgs(()));
            }
            Err(ParseAbilityError::UnknownCommand(_)) => (),
        }

        match read::Ready::try_parse(cmd, args.clone()) {
            Ok(read) => return Ok(Ready::Read(read)),
            Err(ParseAbilityError::InvalidArgs(_)) => {
                return Err(ParseAbilityError::InvalidArgs(()));
            }
            Err(ParseAbilityError::UnknownCommand(_)) => (),
        }

        match update::Ready::try_parse(cmd, args.clone()) {
            Ok(update) => return Ok(Ready::Update(update)),
            Err(ParseAbilityError::InvalidArgs(_)) => {
                return Err(ParseAbilityError::InvalidArgs(()));
            }
            Err(ParseAbilityError::UnknownCommand(_)) => (),
        }

        match destroy::Ready::try_parse(cmd, args) {
            Ok(destroy) => return Ok(Ready::Destroy(destroy)),
            Err(ParseAbilityError::InvalidArgs(_)) => {
                return Err(ParseAbilityError::InvalidArgs(()));
            }
            Err(ParseAbilityError::UnknownCommand(_)) => (),
        }

        Err(ParseAbilityError::UnknownCommand(cmd.into()))
    }
}
//
// impl Checkable for Builder {
//     type Hierarchy = Parentful<Builder>;
// }

impl ToCommand for Ready {
    fn to_command(&self) -> String {
        match self {
            Ready::Create(create) => create.to_command(),
            Ready::Read(read) => read.to_command(),
            Ready::Update(update) => update.to_command(),
            Ready::Destroy(destroy) => destroy.to_command(),
        }
    }
}

impl ToCommand for Promised {
    fn to_command(&self) -> String {
        match self {
            Promised::Create(create) => create.to_command(),
            Promised::Read(read) => read.to_command(),
            Promised::Update(update) => update.to_command(),
            Promised::Destroy(destroy) => destroy.to_command(),
        }
    }
}

// impl ToCommand for Builder {
//     fn to_command(&self) -> String {
//         match self {
//             Builder::Create(create) => create.to_command(),
//             Builder::Read(read) => read.to_command(),
//             Builder::Update(update) => update.to_command(),
//             Builder::Destroy(destroy) => destroy.to_command(),
//         }
//     }
// }
//
// impl CheckParents for Builder {
//     type Parents = MutableParents;
//     type ParentError = (); // FIXME
//
//     fn check_parent(&self, parents: &MutableParents) -> Result<(), Self::ParentError> {
//         match self {
//             Builder::Create(create) => create.check_parent(parents.into()).map_err(|_| ()),
//             Builder::Update(update) => update.check_parent(parents.into()).map_err(|_| ()),
//             Builder::Destroy(destroy) => destroy.check_parent(parents.into()).map_err(|_| ()),
//             Builder::Read(read) => match parents {
//                 MutableParents::Any(crud_any) => read.check_parent(crud_any).map_err(|_| ()),
//                 _ => Err(()),
//             },
//         }
//     }
// }
//
// impl From<Builder> for arguments::Named<Ipld> {
//     fn from(builder: Builder) -> Self {
//         match builder {
//             Builder::Create(create) => create.into(),
//             Builder::Read(read) => read.into(),
//             Builder::Update(update) => update.into(),
//             Builder::Destroy(destroy) => destroy.into(),
//         }
//     }
// }

// impl From<Promised> for arguments::Named<Ipld> {
//     fn from(promised: Promised) -> Self {
//         match promised {
//             Promised::Create(create) => create.into(),
//             Promised::Read(read) => read.into(),
//             Promised::Update(update) => update.into(),
//             Promised::Destroy(destroy) => destroy.into(),
//         }
//     }
// }

// impl From<Ready> for Builder {
//     fn from(ready: Ready) -> Self {
//         match ready {
//             Ready::Create(create) => Builder::Create(create.into()),
//             Ready::Read(read) => Builder::Read(read.into()),
//             Ready::Update(update) => Builder::Update(update.into()),
//             Ready::Destroy(destroy) => Builder::Destroy(destroy.into()),
//         }
//     }
// }
//
// impl TryFrom<Builder> for Ready {
//     type Error = (); // FIXME
//
//     fn try_from(builder: Builder) -> Result<Self, Self::Error> {
//         match builder {
//             Builder::Create(create) => create.try_into().map(Ready::Create).map_err(|_| ()),
//             Builder::Read(read) => read.try_into().map(Ready::Read).map_err(|_| ()),
//             Builder::Update(update) => update.try_into().map(Ready::Update).map_err(|_| ()),
//             Builder::Destroy(destroy) => destroy.try_into().map(Ready::Destroy).map_err(|_| ()),
//         }
//     }
// }
//
// impl CheckSame for Builder {
//     type Error = ();
//
//     fn check_same(&self, other: &Self) -> Result<(), Self::Error> {
//         match (self, other) {
//             (Builder::Create(a), Builder::Create(b)) => a.check_same(b),
//             (Builder::Read(a), Builder::Read(b)) => a.check_same(b),
//             (Builder::Update(a), Builder::Update(b)) => a.check_same(b),
//             (Builder::Destroy(a), Builder::Destroy(b)) => a.check_same(b),
//             _ => Err(()),
//         }
//     }
// }

impl Resolvable for Ready {
    type Promised = Promised;
}

impl From<Promised> for arguments::Named<ipld::Promised> {
    fn from(promised: Promised) -> Self {
        match promised {
            Promised::Create(create) => create.into(),
            Promised::Read(read) => read.into(),
            Promised::Update(update) => update.into(),
            Promised::Destroy(destroy) => destroy.into(),
        }
    }
}
