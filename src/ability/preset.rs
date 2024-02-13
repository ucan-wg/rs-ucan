use super::{msg, wasm};
use crate::{
    ability::{
        arguments,
        command::{Command, ParseAbility},
    },
    delegation::Delegable,
    invocation::{promise, Resolvable},
    proof::{
        checkable::Checkable, parentful::Parentful, parentless::NoParents, parents::CheckParents,
        same::CheckSame,
    },
};
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq)] //, Serialize, Deserialize)]
pub enum Ready {
    //Crud(),
    Msg(msg::Ready),
    Wasm(wasm::run::Ready),
}

#[derive(Debug, Clone, PartialEq)] //, Serialize, Deserialize)]
pub enum Builder {
    Msg(msg::Builder),
    Wasm(wasm::run::Builder),
}

pub enum Parents {
    Msg(msg::Any),
} // NOTE WasmRun has no parents

impl CheckSame for Parents {
    type Error = (); // FIXME

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        match (self, proof) {
            (Parents::Msg(self_), Parents::Msg(proof)) => self_.check_same(proof),
        }
    }
}

impl ParseAbility for Parents {
    type Error = String; // FIXME

    fn try_parse(cmd: &str, args: &arguments::Named<Ipld>) -> Result<Self, Self::Error> {
        todo!()
        // FIXME Ok(Self {})
    }
}

impl Delegable for Ready {
    type Builder = Builder;
}

impl CheckParents for Builder {
    type Parents = Parents;
    type ParentError = (); // FIXME

    fn check_parent(&self, proof: &Parents) -> Result<(), Self::ParentError> {
        match self {
            Builder::Msg(builder) => builder.check_parent(proof),
            Builder::Wasm(builder) => Ok(()),
        }
    }
}

impl Checkable for Builder {
    type Hierarchy = Parentful<Builder>;
}

impl From<Ready> for Builder {
    fn from(ready: Ready) -> Self {
        match ready {
            Ready::Msg(ready) => Builder::Msg(ready.into()),
            Ready::Wasm(ready) => Builder::Wasm(ready.into()),
        }
    }
}

impl TryFrom<Builder> for Ready {
    type Error = (); // FIXME

    fn try_from(builder: Builder) -> Result<Self, Self::Error> {
        match builder {
            Builder::Msg(builder) => builder.try_into().map(Ready::Msg),
            Builder::Wasm(builder) => builder.try_into().map(Ready::Wasm),
        }
    }
}

#[derive(Debug, Clone, PartialEq)] //, Serialize, Deserialize)]
pub enum Promised {
    Msg(msg::Promised),
    Wasm(wasm::run::Promised),
}

impl From<Promised> for arguments::Named<Ipld> {
    fn from(promised: Promised) -> Self {
        match promised {
            Promised::Msg(promised) => promised.into(),
            Promised::Wasm(promised) => promised.into(),
        }
    }
}

impl Resolvable for Ready {
    type Promised = Promised;

    fn try_resolve(promised: Self::Promised) -> Result<Self, Self::Promised> {
        match promised {
            Promised::Msg(promised) => Resolvable::try_resolve(promised)
                .map(Ready::Msg)
                .map_err(Promised::Msg),
            Promised::Wasm(promised) => Resolvable::try_resolve(promised)
                .map(Ready::Wasm)
                .map_err(Promised::Wasm),
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
