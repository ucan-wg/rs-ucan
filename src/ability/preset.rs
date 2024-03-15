use super::{
    crud::{self, Crud, PromisedCrud},
    msg::{Msg, PromisedMsg},
    ucan::revoke::{PromisedRevoke, Revoke},
    wasm::run as wasm,
};
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
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Preset {
    Crud(Crud),
    Msg(Msg),
    Ucan(Revoke),
    Wasm(wasm::Run),
}

impl<T> From<T> for Preset
where
    Crud: From<T>,
{
    fn from(t: T) -> Self {
        Preset::Crud(Crud::from(t))
    }
}

impl ToCommand for Preset {
    fn to_command(&self) -> String {
        match self {
            Preset::Crud(crud) => crud.to_command(),
            Preset::Msg(msg) => msg.to_command(),
            Preset::Ucan(ucan) => ucan.to_command(),
            Preset::Wasm(wasm) => wasm.to_command(),
        }
    }
}

impl From<Preset> for arguments::Named<Ipld> {
    fn from(preset: Preset) -> Self {
        match preset {
            Preset::Crud(crud) => crud.into(),
            Preset::Msg(msg) => msg.into(),
            Preset::Ucan(ucan) => ucan.into(),
            Preset::Wasm(wasm) => wasm.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq)] //, Serialize, Deserialize)]
pub enum PromisedPreset {
    Crud(PromisedCrud),
    Msg(PromisedMsg),
    Ucan(PromisedRevoke),
    Wasm(wasm::PromisedRun),
}

impl Resolvable for Preset {
    type Promised = PromisedPreset;
}

impl ToCommand for PromisedPreset {
    fn to_command(&self) -> String {
        match self {
            PromisedPreset::Crud(promised) => promised.to_command(),
            PromisedPreset::Msg(promised) => promised.to_command(),
            PromisedPreset::Ucan(promised) => promised.to_command(),
            PromisedPreset::Wasm(promised) => promised.to_command(),
        }
    }
}

impl ParsePromised for PromisedPreset {
    type PromisedArgsError = ParsePromisedError;

    fn try_parse_promised(
        cmd: &str,
        args: arguments::Named<ipld::Promised>,
    ) -> Result<Self, ParseAbilityError<Self::PromisedArgsError>> {
        match PromisedCrud::try_parse_promised(cmd, args.clone()) {
            Ok(promised) => return Ok(PromisedPreset::Crud(promised)),
            Err(ParseAbilityError::UnknownCommand(_)) => (),
            Err(err) => {
                return Err(ParseAbilityError::InvalidArgs(ParsePromisedError::Crud(
                    err,
                )))
            }
        }

        match PromisedMsg::try_parse_promised(cmd, args.clone()) {
            Ok(promised) => return Ok(PromisedPreset::Msg(promised)),
            Err(ParseAbilityError::UnknownCommand(_)) => (),
            Err(_err) => return Err(ParseAbilityError::InvalidArgs(ParsePromisedError::Msg)),
        }

        match wasm::PromisedRun::try_parse_promised(cmd, args.clone()) {
            Ok(promised) => return Ok(PromisedPreset::Wasm(promised)),
            Err(ParseAbilityError::UnknownCommand(_)) => (),
            Err(_err) => return Err(ParseAbilityError::InvalidArgs(ParsePromisedError::Wasm)),
        }

        match PromisedRevoke::try_parse_promised(cmd, args) {
            Ok(promised) => return Ok(PromisedPreset::Ucan(promised)),
            Err(ParseAbilityError::UnknownCommand(_)) => (),
            Err(_err) => return Err(ParseAbilityError::InvalidArgs(ParsePromisedError::Ucan)),
        }

        Err(ParseAbilityError::UnknownCommand(cmd.to_string()))
    }
}

#[derive(Debug, Clone, Error)]
pub enum ParsePromisedError {
    #[error("Crud error: {0}")]
    Crud(ParseAbilityError<crud::InvalidArgs>),

    #[error("Msg error")]
    Msg, // FIXME

    #[error("Wasm error")]
    Wasm, // FIXME

    #[error("Ucan error")]
    Ucan, // FIXME
}

impl ParseAbility for Preset {
    type ArgsErr = ();

    fn try_parse(
        cmd: &str,
        args: arguments::Named<Ipld>,
    ) -> Result<Self, ParseAbilityError<Self::ArgsErr>> {
        match Msg::try_parse(cmd, args.clone()) {
            Ok(msg) => return Ok(Preset::Msg(msg)),
            Err(ParseAbilityError::UnknownCommand(_)) => (), // FIXME
            Err(err) => return Err(err),
        }

        match Crud::try_parse(cmd, args.clone()) {
            Ok(crud) => return Ok(Preset::Crud(crud)),
            Err(ParseAbilityError::UnknownCommand(_)) => (),
            Err(err) => return Err(err),
        }

        match wasm::Run::try_parse(cmd, args.clone()) {
            Ok(wasm) => return Ok(Preset::Wasm(wasm)),
            Err(ParseAbilityError::UnknownCommand(_)) => (),
            Err(err) => return Err(err),
        }

        match Revoke::try_parse(cmd, args) {
            Ok(ucan) => return Ok(Preset::Ucan(ucan)),
            Err(ParseAbilityError::UnknownCommand(_)) => (),
            Err(err) => return Err(err),
        }

        Err(ParseAbilityError::UnknownCommand(cmd.to_string()))
    }
}

impl From<PromisedPreset> for arguments::Named<ipld::Promised> {
    fn from(promised: PromisedPreset) -> Self {
        match promised {
            PromisedPreset::Crud(promised) => promised.into(),
            PromisedPreset::Msg(promised) => promised.into(),
            PromisedPreset::Ucan(promised) => promised.into(),
            PromisedPreset::Wasm(promised) => promised.into(),
        }
    }
}
