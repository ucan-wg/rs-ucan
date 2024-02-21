use super::{crud, msg, wasm};
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

#[derive(Debug, Clone, PartialEq)] //, Serialize, Deserialize)]
pub enum Ready {
    // FIXME UCAN
    Crud(crud::Ready),
    Msg(msg::Ready),
    Wasm(wasm::run::Ready),
}

#[derive(Debug, Clone, PartialEq)] //, Serialize, Deserialize)]
pub enum Builder {
    Crud(crud::Builder),
    Msg(msg::Builder),
    Wasm(wasm::run::Builder),
}

#[derive(Debug, Clone, PartialEq)] //, Serialize, Deserialize)]
pub enum Parents {
    Crud(crud::MutableParents),
    Msg(msg::Any),
} // NOTE WasmRun has no parents

impl CheckSame for Parents {
    type Error = (); // FIXME

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        match (self, proof) {
            (Parents::Msg(self_), Parents::Msg(proof_)) => self_.check_same(proof_).map_err(|_| ()),
            (Parents::Crud(self_), Parents::Crud(proof_)) => {
                self_.check_same(proof_).map_err(|_| ())
            }
            _ => Err(()),
        }
    }
}

impl Delegable for Ready {
    type Builder = Builder;
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

impl ToCommand for Builder {
    fn to_command(&self) -> String {
        match self {
            Builder::Crud(builder) => builder.to_command(),
            Builder::Msg(builder) => builder.to_command(),
            Builder::Wasm(builder) => builder.to_command(),
        }
    }
}

impl CheckSame for Builder {
    type Error = (); // FIXME

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        match (self, proof) {
            (Builder::Wasm(builder), Builder::Wasm(proof)) => builder.check_same(proof),
            _ => Err(()),
        }
    }
}

impl CheckParents for Builder {
    type Parents = Parents;
    type ParentError = (); // FIXME

    fn check_parent(&self, proof: &Self::Parents) -> Result<(), Self::ParentError> {
        match (self, proof) {
            (Builder::Msg(builder), Parents::Msg(proof)) => builder.check_parent(proof),
            _ => Err(()),
        }
    }
}

impl Checkable for Builder {
    type Hierarchy = Parentful<Builder>;
}

impl From<Ready> for Builder {
    fn from(ready: Ready) -> Self {
        match ready {
            Ready::Crud(ready) => Builder::Crud(ready.into()),
            Ready::Msg(ready) => Builder::Msg(ready.into()),
            Ready::Wasm(ready) => Builder::Wasm(ready.into()),
        }
    }
}

impl TryFrom<Builder> for Ready {
    type Error = (); // FIXME

    fn try_from(builder: Builder) -> Result<Self, Self::Error> {
        match builder {
            Builder::Crud(builder) => builder.try_into().map(Ready::Crud).map_err(|_| ()),
            Builder::Msg(builder) => builder.try_into().map(Ready::Msg).map_err(|_| ()),
            Builder::Wasm(builder) => builder.try_into().map(Ready::Wasm).map_err(|_| ()),
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

impl ParseAbility for Builder {
    type ArgsErr = ();

    fn try_parse(
        cmd: &str,
        args: arguments::Named<Ipld>,
    ) -> Result<Self, ParseAbilityError<Self::ArgsErr>> {
        match msg::Builder::try_parse(cmd, args.clone()) {
            Ok(builder) => return Ok(Builder::Msg(builder)),
            Err(err) => return Err(err),
            Err(ParseAbilityError::UnknownCommand(_)) => (),
        }

        match crud::Builder::try_parse(cmd, args.clone()) {
            Ok(builder) => return Ok(Builder::Crud(builder)),
            Err(err) => return Err(err),
            Err(ParseAbilityError::UnknownCommand(_)) => (),
        }

        match wasm::run::Builder::try_parse(cmd, args) {
            Ok(builder) => return Ok(Builder::Wasm(builder)),
            Err(err) => return Err(err),
            Err(ParseAbilityError::UnknownCommand(_)) => (),
        }

        Err(ParseAbilityError::UnknownCommand(cmd.to_string()))
    }
}

impl From<Builder> for arguments::Named<Ipld> {
    fn from(builder: Builder) -> Self {
        match builder {
            Builder::Crud(builder) => builder.into(),
            Builder::Msg(builder) => builder.into(),
            Builder::Wasm(builder) => builder.into(),
        }
    }
}

impl From<Parents> for arguments::Named<Ipld> {
    fn from(parents: Parents) -> Self {
        match parents {
            Parents::Crud(parents) => parents.into(),
            Parents::Msg(parents) => parents.into(),
        }
    }
}

impl From<Promised> for arguments::Named<ipld::Promised> {
    fn from(promised: Promised) -> Self {
        match promised {
            Promised::Crud(promised) => promised.into(),
            Promised::Msg(promised) => promised.into(),
            Promised::Wasm(promised) => promised.into(),
        }
    }
}
