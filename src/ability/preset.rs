use super::{crud, msg, wasm};
use crate::{
    ability::{arguments, command::ParseAbility},
    delegation::Delegable,
    invocation::Resolvable,
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

impl ParseAbility for Parents {
    type Error = String; // FIXME

    fn try_parse(cmd: &str, args: &arguments::Named<Ipld>) -> Result<Self, Self::Error> {
        if let Ok(crud) = crud::MutableParents::try_parse(cmd, args) {
            return Ok(Parents::Crud(crud));
        }

        if let Ok(msg) = msg::Any::try_parse(cmd, args) {
            return Ok(Parents::Msg(msg));
        }

        Err("Nope".into())
    }
}

impl Delegable for Ready {
    type Builder = Builder;
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

impl From<Promised> for arguments::Named<Ipld> {
    fn from(promised: Promised) -> Self {
        match promised {
            Promised::Crud(promised) => promised.into(),
            Promised::Msg(promised) => promised.into(),
            Promised::Wasm(promised) => promised.into(),
        }
    }
}

impl Resolvable for Ready {
    type Promised = Promised;

    fn try_resolve(promised: Self::Promised) -> Result<Self, Self::Promised> {
        match promised {
            Promised::Crud(promised) => Resolvable::try_resolve(promised)
                .map(Ready::Crud)
                .map_err(Promised::Crud),
            Promised::Msg(promised) => Resolvable::try_resolve(promised)
                .map(Ready::Msg)
                .map_err(Promised::Msg),
            Promised::Wasm(promised) => Resolvable::try_resolve(promised)
                .map(Ready::Wasm)
                .map_err(Promised::Wasm),
        }
    }
}

impl From<Promised> for Builder {
    fn from(promised: Promised) -> Self {
        match promised {
            Promised::Crud(promised) => Builder::Crud(promised.into()),
            Promised::Msg(promised) => Builder::Msg(promised.into()),
            Promised::Wasm(promised) => Builder::Wasm(promised.into()),
        }
    }
}
