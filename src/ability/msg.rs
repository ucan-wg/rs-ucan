//! Message abilities

pub mod receive;
pub mod send;

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

// FIXME rename invokable?
#[derive(Debug, Clone, PartialEq)]
pub enum Ready {
    Send(send::Ready),
    Receive(receive::Receive),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Promised {
    Send(send::Promised),
    Receive(receive::Promised),
}

impl ToCommand for Ready {
    fn to_command(&self) -> String {
        match self {
            Ready::Send(send) => send.to_command(),
            Ready::Receive(receive) => receive.to_command(),
        }
    }
}

impl ToCommand for Promised {
    fn to_command(&self) -> String {
        match self {
            Promised::Send(send) => send.to_command(),
            Promised::Receive(receive) => receive.to_command(),
        }
    }
}

impl ParsePromised for Promised {
    type PromisedArgsError = ();

    fn try_parse_promised(
        cmd: &str,
        args: arguments::Named<ipld::Promised>,
    ) -> Result<Self, ParseAbilityError<Self::PromisedArgsError>> {
        if let Ok(send) = send::Promised::try_parse_promised(cmd, args.clone()) {
            return Ok(Promised::Send(send));
        }

        if let Ok(receive) = receive::Promised::try_parse_promised(cmd, args) {
            return Ok(Promised::Receive(receive));
        }

        Err(ParseAbilityError::UnknownCommand(cmd.to_string()))
    }
}

impl From<Promised> for arguments::Named<Ipld> {
    fn from(promised: Promised) -> Self {
        match promised {
            Promised::Send(send) => send.into(),
            Promised::Receive(receive) => receive.into(),
        }
    }
}

impl Resolvable for Ready {
    type Promised = Promised;
}

impl From<Promised> for arguments::Named<ipld::Promised> {
    fn from(promised: Promised) -> Self {
        match promised {
            Promised::Send(send) => send.into(),
            Promised::Receive(receive) => receive.into(),
        }
    }
}

impl ParseAbility for Ready {
    type ArgsErr = ();

    fn try_parse(
        cmd: &str,
        args: arguments::Named<Ipld>,
    ) -> Result<Self, ParseAbilityError<Self::ArgsErr>> {
        if let Ok(send) = send::Ready::try_parse(cmd, args.clone()) {
            return Ok(Ready::Send(send));
        }

        if let Ok(receive) = receive::Receive::try_parse(cmd, args) {
            return Ok(Ready::Receive(receive));
        }

        Err(ParseAbilityError::UnknownCommand(cmd.to_string()))
    }
}
