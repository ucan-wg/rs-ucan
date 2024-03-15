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

pub mod create;
pub mod destroy;
pub mod read;
pub mod update;

use crate::{
    ability::{
        arguments,
        command::ToCommand,
        parse::{ParseAbility, ParseAbilityError, ParsePromised},
    },
    invocation::promise::Resolvable,
    ipld,
};
use create::{Create, PromisedCreate};
use destroy::{Destroy, PromisedDestroy};
use libipld_core::ipld::Ipld;
use read::{PromisedRead, Read};
use serde::{Deserialize, Serialize};
use thiserror::Error;
use update::{PromisedUpdate, Update};

#[cfg(target_arch = "wasm32")]
pub mod js;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Crud {
    Create(Create),
    Read(Read),
    Update(Update),
    Destroy(Destroy),
}

impl From<Crud> for arguments::Named<Ipld> {
    fn from(crud: Crud) -> Self {
        match crud {
            Crud::Create(create) => create.into(),
            Crud::Read(read) => read.into(),
            Crud::Update(update) => update.into(),
            Crud::Destroy(destroy) => destroy.into(),
        }
    }
}

impl From<Create> for Crud {
    fn from(create: Create) -> Self {
        Crud::Create(create)
    }
}

impl From<Read> for Crud {
    fn from(read: Read) -> Self {
        Crud::Read(read)
    }
}

impl From<Update> for Crud {
    fn from(update: Update) -> Self {
        Crud::Update(update)
    }
}

impl From<Destroy> for Crud {
    fn from(destroy: Destroy) -> Self {
        Crud::Destroy(destroy)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PromisedCrud {
    Create(PromisedCreate),
    Read(PromisedRead),
    Update(PromisedUpdate),
    Destroy(PromisedDestroy),
}

impl ParsePromised for PromisedCrud {
    type PromisedArgsError = InvalidArgs;

    fn try_parse_promised(
        cmd: &str,
        args: arguments::Named<ipld::Promised>,
    ) -> Result<Self, ParseAbilityError<Self::PromisedArgsError>> {
        match PromisedCreate::try_parse_promised(cmd, args.clone()) {
            Ok(create) => return Ok(PromisedCrud::Create(create)),
            Err(ParseAbilityError::InvalidArgs(e)) => {
                return Err(ParseAbilityError::InvalidArgs(InvalidArgs::Create(e)))
            }
            Err(ParseAbilityError::UnknownCommand(_)) => (),
        }

        match PromisedRead::try_parse_promised(cmd, args.clone()) {
            Ok(read) => return Ok(PromisedCrud::Read(read)),
            Err(ParseAbilityError::InvalidArgs(e)) => {
                return Err(ParseAbilityError::InvalidArgs(InvalidArgs::Read(e)))
            }
            Err(ParseAbilityError::UnknownCommand(_)) => (),
        }

        match PromisedUpdate::try_parse_promised(cmd, args.clone()) {
            Ok(update) => return Ok(PromisedCrud::Update(update)),
            Err(ParseAbilityError::InvalidArgs(e)) => {
                return Err(ParseAbilityError::InvalidArgs(InvalidArgs::Update(e)))
            }
            Err(ParseAbilityError::UnknownCommand(_)) => (),
        }

        match PromisedDestroy::try_parse_promised(cmd, args) {
            Ok(destroy) => return Ok(PromisedCrud::Destroy(destroy)),
            Err(ParseAbilityError::InvalidArgs(e)) => {
                return Err(ParseAbilityError::InvalidArgs(InvalidArgs::Destroy(e)))
            }
            Err(ParseAbilityError::UnknownCommand(_)) => (),
        }

        Err(ParseAbilityError::UnknownCommand(cmd.into()))
    }
}

#[derive(Debug, Clone, PartialEq, Error)]
pub enum InvalidArgs {
    #[error("Invalid args for create: {0}")]
    Create(create::FromPromisedArgsError),

    #[error("Invalid args for read: {0}")]
    Read(read::FromPromisedArgsError),

    #[error("Invalid args for update: {0}")]
    Update(update::FromPromisedArgsError),

    #[error("Invalid args for destroy: {0}")]
    Destroy(destroy::FromPromisedArgsError),
}

impl ParseAbility for Crud {
    type ArgsErr = ();

    fn try_parse(
        cmd: &str,
        args: arguments::Named<Ipld>,
    ) -> Result<Self, ParseAbilityError<Self::ArgsErr>> {
        match Create::try_parse(cmd, args.clone()) {
            Ok(create) => return Ok(Crud::Create(create)),
            Err(ParseAbilityError::InvalidArgs(_)) => {
                return Err(ParseAbilityError::InvalidArgs(()));
            }
            Err(ParseAbilityError::UnknownCommand(_)) => (),
        }

        match Read::try_parse(cmd, args.clone()) {
            Ok(read) => return Ok(Crud::Read(read)),
            Err(ParseAbilityError::InvalidArgs(_)) => {
                return Err(ParseAbilityError::InvalidArgs(()));
            }
            Err(ParseAbilityError::UnknownCommand(_)) => (),
        }

        match Update::try_parse(cmd, args.clone()) {
            Ok(update) => return Ok(Crud::Update(update)),
            Err(ParseAbilityError::InvalidArgs(_)) => {
                return Err(ParseAbilityError::InvalidArgs(()));
            }
            Err(ParseAbilityError::UnknownCommand(_)) => (),
        }

        match Destroy::try_parse(cmd, args) {
            Ok(destroy) => return Ok(Crud::Destroy(destroy)),
            Err(ParseAbilityError::InvalidArgs(_)) => {
                return Err(ParseAbilityError::InvalidArgs(()));
            }
            Err(ParseAbilityError::UnknownCommand(_)) => (),
        }

        Err(ParseAbilityError::UnknownCommand(cmd.into()))
    }
}

impl ToCommand for Crud {
    fn to_command(&self) -> String {
        match self {
            Crud::Create(create) => create.to_command(),
            Crud::Read(read) => read.to_command(),
            Crud::Update(update) => update.to_command(),
            Crud::Destroy(destroy) => destroy.to_command(),
        }
    }
}

impl ToCommand for PromisedCrud {
    fn to_command(&self) -> String {
        match self {
            PromisedCrud::Create(create) => create.to_command(),
            PromisedCrud::Read(read) => read.to_command(),
            PromisedCrud::Update(update) => update.to_command(),
            PromisedCrud::Destroy(destroy) => destroy.to_command(),
        }
    }
}
impl Resolvable for Crud {
    type Promised = PromisedCrud;
}

impl From<PromisedCrud> for arguments::Named<ipld::Promised> {
    fn from(promised: PromisedCrud) -> Self {
        match promised {
            PromisedCrud::Create(create) => create.into(),
            PromisedCrud::Read(read) => read.into(),
            PromisedCrud::Update(update) => update.into(),
            PromisedCrud::Destroy(destroy) => destroy.into(),
        }
    }
}
