//! Message abilities

mod any;

pub mod receive;
pub mod send;

pub use any::Any;

use crate::{
    ability::{
        arguments,
        command::{ParseAbility, ParseAbilityError, ToCommand},
    },
    delegation::Delegable,
    invocation::promise::Resolvable,
    ipld,
    proof::{checkable::Checkable, parentful::Parentful, parents::CheckParents, same::CheckSame},
};
use libipld_core::ipld::Ipld;

// FIXME rename invokable?
#[derive(Debug, Clone, PartialEq)]
pub enum Ready {
    Send(send::Ready),
    Receive(receive::Receive),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Builder {
    Send(send::Builder),
    Receive(receive::Receive),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Promised {
    Send(send::Promised),
    Receive(receive::Promised),
}

impl Delegable for Ready {
    type Builder = Builder;
}

impl ToCommand for Ready {
    fn to_command(&self) -> String {
        match self {
            Ready::Send(send) => send.to_command(),
            Ready::Receive(receive) => receive.to_command(),
        }
    }
}

impl ToCommand for Builder {
    fn to_command(&self) -> String {
        match self {
            Builder::Send(send) => send.to_command(),
            Builder::Receive(receive) => receive.to_command(),
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

// impl ParseAbility for Ready {
//     type ArgsErr = ();
//
//     fn try_parse(
//         cmd: &str,
//         args: arguments::Named<Ipld>,
//     ) -> Result<Self, ParseAbilityError<Self::ArgsErr>> {
//         match send::Ready::try_parse(cmd, args.clone()) {
//             Ok(send) => return Ok(Ready::Send(send)),
//             Err(ParseAbilityError::InvalidArgs(args)) => {
//                 return Err(ParseAbilityError::InvalidArgs(()))
//             }
//             Err(ParseAbilityError::UnknownCommand(_)) => {}
//         }
//
//         match receive::Receive::try_parse(cmd, args) {
//             Ok(receive) => return Ok(Ready::Receive(receive)),
//             Err(ParseAbilityError::InvalidArgs(args)) => {
//                 return Err(ParseAbilityError::InvalidArgs(()))
//             }
//             Err(ParseAbilityError::UnknownCommand(cmd)) => {}
//         }
//
//         Err(ParseAbilityError::UnknownCommand(cmd.to_string()))
//     }
// }

impl ParseAbility for Builder {
    type ArgsErr = ();

    fn try_parse(
        cmd: &str,
        args: arguments::Named<Ipld>,
    ) -> Result<Self, ParseAbilityError<Self::ArgsErr>> {
        if let Ok(send) = send::Builder::try_parse(cmd, args.clone()) {
            return Ok(Builder::Send(send));
        }

        if let Ok(receive) = receive::Receive::try_parse(cmd, args) {
            return Ok(Builder::Receive(receive));
        }

        Err(ParseAbilityError::UnknownCommand(cmd.to_string()))
    }
}

impl TryFrom<Builder> for Ready {
    type Error = ();

    fn try_from(builder: Builder) -> Result<Self, Self::Error> {
        match builder {
            Builder::Send(send) => send.try_into().map(Ready::Send).map_err(|_| ()),
            Builder::Receive(receive) => Ok(Ready::Receive(receive)),
        }
    }
}

impl From<Ready> for Builder {
    fn from(ready: Ready) -> Self {
        match ready {
            Ready::Send(send) => Builder::Send(send.into()),
            Ready::Receive(receive) => Builder::Receive(receive.into()),
        }
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

impl CheckSame for Builder {
    type Error = (); // FIXME

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        match (self, proof) {
            (Builder::Send(this), Builder::Send(that)) => this.check_same(that),
            (Builder::Receive(this), Builder::Receive(that)) => this.check_same(that),
            _ => Err(()),
        }
    }
}

impl CheckParents for Builder {
    type Parents = Any;
    type ParentError = ();

    fn check_parent(&self, proof: &Any) -> Result<(), Self::ParentError> {
        match (self, proof) {
            (Builder::Send(this), any) => this.check_parent(&any),
            (Builder::Receive(this), any) => this.check_parent(&any),
        }
    }
}

impl Checkable for Builder {
    type Hierarchy = Parentful<Builder>;
}

impl From<Builder> for arguments::Named<Ipld> {
    fn from(builder: Builder) -> Self {
        match builder {
            Builder::Send(send) => send.into(),
            Builder::Receive(receive) => receive.into(),
        }
    }
}

impl From<Promised> for arguments::Named<ipld::Promised> {
    fn from(promised: Promised) -> Self {
        match promised {
            Promised::Send(send) => send.into(),
            Promised::Receive(receive) => receive.into(),
        }
    }
}
