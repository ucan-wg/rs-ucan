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
use receive::{PromisedReceive, Receive};
use send::{PromisedSend, Send};
use serde::{Deserialize, Serialize};

/// A family of abilities for sending and receiving messages.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Msg {
    /// The ability for sending messages.
    Send(Send),

    /// The ability for receiving messages.
    Receive(Receive),
}

/// A promised version of the [`Msg`] ability.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PromisedMsg {
    /// The promised ability for sending messages.
    Send(PromisedSend),

    /// The promised ability for receiving messages.
    Receive(PromisedReceive),
}

impl ToCommand for Msg {
    fn to_command(&self) -> String {
        match self {
            Msg::Send(send) => send.to_command(),
            Msg::Receive(receive) => receive.to_command(),
        }
    }
}

impl ToCommand for PromisedMsg {
    fn to_command(&self) -> String {
        match self {
            PromisedMsg::Send(send) => send.to_command(),
            PromisedMsg::Receive(receive) => receive.to_command(),
        }
    }
}

impl ParsePromised for PromisedMsg {
    type PromisedArgsError = ();

    fn try_parse_promised(
        cmd: &str,
        args: arguments::Named<ipld::Promised>,
    ) -> Result<Self, ParseAbilityError<Self::PromisedArgsError>> {
        if let Ok(send) = PromisedSend::try_parse_promised(cmd, args.clone()) {
            return Ok(PromisedMsg::Send(send));
        }

        if let Ok(receive) = PromisedReceive::try_parse_promised(cmd, args) {
            return Ok(PromisedMsg::Receive(receive));
        }

        Err(ParseAbilityError::UnknownCommand(cmd.to_string()))
    }
}

impl From<PromisedMsg> for arguments::Named<Ipld> {
    fn from(promised: PromisedMsg) -> Self {
        match promised {
            PromisedMsg::Send(send) => send.into(),
            PromisedMsg::Receive(receive) => receive.into(),
        }
    }
}

impl Resolvable for Msg {
    type Promised = PromisedMsg;
}

impl From<PromisedMsg> for arguments::Named<ipld::Promised> {
    fn from(promised: PromisedMsg) -> Self {
        match promised {
            PromisedMsg::Send(send) => send.into(),
            PromisedMsg::Receive(receive) => receive.into(),
        }
    }
}

impl ParseAbility for Msg {
    type ArgsErr = ();

    fn try_parse(
        cmd: &str,
        args: arguments::Named<Ipld>,
    ) -> Result<Self, ParseAbilityError<Self::ArgsErr>> {
        if let Ok(send) = Send::try_parse(cmd, args.clone()) {
            return Ok(Msg::Send(send));
        }

        if let Ok(receive) = Receive::try_parse(cmd, args) {
            return Ok(Msg::Receive(receive));
        }

        Err(ParseAbilityError::UnknownCommand(cmd.to_string()))
    }
}
