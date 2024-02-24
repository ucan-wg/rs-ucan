use super::{crud, msg, wasm};
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

#[derive(Debug, Clone, PartialEq)] //, Serialize, Deserialize)]
pub enum Ready {
    // FIXME UCAN
    Crud(crud::Ready),
    Msg(msg::Ready),
    Wasm(wasm::run::Ready),
}

impl ToCommand for Ready {
    fn to_command(&self) -> String {
        match self {
            Ready::Crud(ready) => ready.to_command(),
            Ready::Msg(ready) => ready.to_command(),
            Ready::Wasm(ready) => ready.to_command(),
        }
    }
}
#[derive(Debug, Clone, PartialEq)] //, Serialize, Deserialize)]
pub enum Promised {
    Crud(crud::Promised),
    Msg(msg::Promised),
    Wasm(wasm::run::Promised),
}

impl Resolvable for Ready {
    type Promised = Promised;
}

impl ToCommand for Promised {
    fn to_command(&self) -> String {
        match self {
            Promised::Crud(promised) => promised.to_command(),
            Promised::Msg(promised) => promised.to_command(),
            Promised::Wasm(promised) => promised.to_command(),
        }
    }
}

impl ParsePromised for Promised {
    type PromisedArgsError = ();

    fn try_parse_promised(
        cmd: &str,
        args: arguments::Named<ipld::Promised>,
    ) -> Result<Self, ParseAbilityError<Self::PromisedArgsError>> {
        match crud::Promised::try_parse_promised(cmd, args.clone()) {
            Ok(promised) => return Ok(Promised::Crud(promised)),
            Err(err) => return Err(err),
            Err(ParseAbilityError::UnknownCommand(_)) => (),
        }

        match msg::Promised::try_parse_promised(cmd, args.clone()) {
            Ok(promised) => return Ok(Promised::Msg(promised)),
            Err(err) => return Err(err),
            Err(ParseAbilityError::UnknownCommand(_)) => (),
        }

        match wasm::run::Promised::try_parse_promised(cmd, args) {
            Ok(promised) => return Ok(Promised::Wasm(promised)),
            Err(err) => return Err(err),
            Err(ParseAbilityError::UnknownCommand(_)) => (),
        }

        Err(ParseAbilityError::UnknownCommand(cmd.to_string()))
    }
}

impl ParseAbility for Ready {
    type ArgsErr = ();

    fn try_parse(
        cmd: &str,
        args: arguments::Named<Ipld>,
    ) -> Result<Self, ParseAbilityError<Self::ArgsErr>> {
        match msg::Ready::try_parse(cmd, args.clone()) {
            Ok(builder) => return Ok(Ready::Msg(builder)),
            Err(err) => return Err(err),
            Err(ParseAbilityError::UnknownCommand(_)) => (),
        }

        match crud::Ready::try_parse(cmd, args.clone()) {
            Ok(builder) => return Ok(Ready::Crud(builder)),
            Err(err) => return Err(err),
            Err(ParseAbilityError::UnknownCommand(_)) => (),
        }

        match wasm::run::Ready::try_parse(cmd, args) {
            Ok(builder) => return Ok(Ready::Wasm(builder)),
            Err(err) => return Err(err),
            Err(ParseAbilityError::UnknownCommand(_)) => (),
        }

        Err(ParseAbilityError::UnknownCommand(cmd.to_string()))
    }
}

// impl From<Builder> for arguments::Named<Ipld> {
//     fn from(builder: Builder) -> Self {
//         match builder {
//             Builder::Crud(builder) => builder.into(),
//             Builder::Msg(builder) => builder.into(),
//             Builder::Wasm(builder) => builder.into(),
//         }
//     }
// }
//
// impl From<Parents> for arguments::Named<Ipld> {
//     fn from(parents: Parents) -> Self {
//         match parents {
//             Parents::Crud(parents) => parents.into(),
//             Parents::Msg(parents) => parents.into(),
//         }
//     }
// }

impl From<Promised> for arguments::Named<ipld::Promised> {
    fn from(promised: Promised) -> Self {
        match promised {
            Promised::Crud(promised) => promised.into(),
            Promised::Msg(promised) => promised.into(),
            Promised::Wasm(promised) => promised.into(),
        }
    }
}
