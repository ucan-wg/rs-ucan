use super::command::Command;
use crate::{ability::arguments, ipld};
use libipld_core::ipld::Ipld;
use std::fmt;
use thiserror::Error;

pub trait ParseAbility: Sized {
    type ArgsErr: fmt::Debug;

    fn try_parse(
        cmd: &str,
        args: arguments::Named<Ipld>,
    ) -> Result<Self, ParseAbilityError<Self::ArgsErr>>;
}

#[derive(Debug, Clone, Error)]
pub enum ParseAbilityError<E> {
    #[error("Unknown command: {0}")]
    UnknownCommand(String),

    #[error(transparent)]
    InvalidArgs(#[from] E),
}

impl<T: Command + TryFrom<arguments::Named<Ipld>>> ParseAbility for T
where
    <T as TryFrom<arguments::Named<Ipld>>>::Error: fmt::Debug,
{
    type ArgsErr = <T as TryFrom<arguments::Named<Ipld>>>::Error;

    fn try_parse(
        cmd: &str,
        args: arguments::Named<Ipld>,
    ) -> Result<Self, ParseAbilityError<<Self as TryFrom<arguments::Named<Ipld>>>::Error>> {
        if cmd != T::COMMAND {
            return Err(ParseAbilityError::UnknownCommand(cmd.to_string()));
        }

        Self::try_from(args).map_err(ParseAbilityError::InvalidArgs)
    }
}

pub trait ParsePromised: Sized {
    type PromisedArgsError;

    fn try_parse_promised(
        cmd: &str,
        args: arguments::Named<ipld::Promised>,
    ) -> Result<Self, ParseAbilityError<Self::PromisedArgsError>>;
}

impl<T: Command + TryFrom<arguments::Named<ipld::Promised>>> ParsePromised for T
where
    <T as TryFrom<arguments::Named<ipld::Promised>>>::Error: fmt::Debug,
{
    type PromisedArgsError = <T as TryFrom<arguments::Named<ipld::Promised>>>::Error;

    fn try_parse_promised(
        cmd: &str,
        args: arguments::Named<ipld::Promised>,
    ) -> Result<Self, ParseAbilityError<Self::PromisedArgsError>> {
        if cmd != T::COMMAND {
            return Err(ParseAbilityError::UnknownCommand(cmd.to_string()));
        }

        Self::try_from(args).map_err(ParseAbilityError::InvalidArgs)
    }
}
