//! Ability command utilities
//!
//! Commands are the `cmd` field of a UCAN, and set the shape of the `args` field.
//!
//! ```js
//! // Here is a UCAN payload:
//! {
//!   "iss": "did:example:123",
//!   "aud": "did:example:456",
//!   "cmd": "msg/send", // <--- This is the command
//!   "args": {                           // ┐
//!     "to": "mailto:alice@example.com", // ├─ These are determined by the command
//!     "message": "Hello, World!",       // │
//!   }                                   // ┘
//!   "exp": 1234567890
//! }
//! ```

use crate::ability::arguments;
use libipld_core::ipld::Ipld;
use std::fmt;
use thiserror::Error;

/// Attach a `cmd` field to a type
///
/// Commands are the `cmd` field of a UCAN, and set the shape of the `args` field.
/// The `COMMAND` attaches this to types so that they can be serialized appropriately.
///
/// # Examples
///
/// ```rust
/// # use ucan::ability::command::Command;
/// #
/// struct Upload {
///    pub gb_quota: u64,
///    pub mime_types: Vec<String>,
/// }
///
/// impl Command for Upload {
///    const COMMAND: &'static str = "storage/upload";
/// }
///
/// assert_eq!(Upload::COMMAND, "storage/upload");
/// ```
pub trait Command {
    /// The value that will be placed in the UCAN's `cmd` field for the given type
    ///
    /// FIXME
    /// This is a `const` because it *must not*[^dynamic] depend on the runtime values of a type
    /// in order to ensure type safety.
    ///
    /// [^dynamic]: <small>Note that if the `dynamic` feature is enabled, the exception is
    /// a special ability called [`Dynamic`][super::dynamic::Dynamic] (for e.g. JS FFI)
    /// that uses a non-exported code path separate from the [`Command`] trait.</small>
    const COMMAND: &'static str;
}

// FIXME definitely needs a better name
pub trait ParseAbility: Sized {
    type Error: fmt::Display;

    // FIXME rename this trait to Ability?
    fn try_parse(cmd: &str, args: &arguments::Named<Ipld>) -> Result<Self, Self::Error>;
}

#[derive(Debug, Clone, Error)]
pub enum ParseAbilityError<E: fmt::Display> {
    #[error("Unknown command")]
    UnknownCommand,

    #[error(transparent)]
    InvalidArgs(#[from] E),
}

impl<T: Command + TryFrom<Ipld>> ParseAbility for T
where
    <T as TryFrom<Ipld>>::Error: fmt::Display,
{
    type Error = ParseAbilityError<<T as TryFrom<Ipld>>::Error>;

    fn try_parse(cmd: &str, args: &arguments::Named<Ipld>) -> Result<Self, Self::Error> {
        if cmd != T::COMMAND {
            return Err(ParseAbilityError::UnknownCommand);
        }

        Ipld::from(args.clone())
            .try_into()
            .map_err(ParseAbilityError::InvalidArgs)
    }
}

// NOTE do not export; this is used to limit the Hierarchy
// interface to [Parentful] and [Parentless] while enabling [Dynamic]
// FIXME ^^^^ NOT ANYMORE
// Either that needs to be re-locked down, or (because it's all abstract anyways)
// just note that you probably don;t want this one.
pub trait ToCommand {
    fn to_command(&self) -> String;
}

impl<T: Command> ToCommand for T {
    fn to_command(&self) -> String {
        T::COMMAND.to_string()
    }
}
