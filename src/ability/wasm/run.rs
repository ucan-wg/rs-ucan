//! Ability to run a Wasm module

use super::module::Module;
use crate::{
    ability::{arguments, command::Command},
    delegation::Delegable,
    invocation::promise,
    proof::{parentless::NoParents, same::CheckSame},
};
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// The ability to run a Wasm module on the subject's machine
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Generic<Mod, Fun, Args> {
    /// The Wasm module to run
    pub module: Mod,

    /// The function from the module to run
    pub function: Fun,

    /// Arguments to pass to the function
    pub args: Args,
}

impl<Mod, Fun, Args> Command for Generic<Mod, Fun, Args> {
    const COMMAND: &'static str = "wasm/run";
}

/// A variant with all of the required fields filled in
pub type Ready = Generic<Module, String, Vec<Ipld>>;

impl Delegable for Ready {
    type Builder = Builder;
}

impl promise::Resolvable for Ready {
    type Promised = Promised;

    fn try_resolve(promised: Self::Promised) -> Result<Self, Self::Promised> {
        match promise::Resolves::try_resolve_3(promised.module, promised.function, promised.args) {
            Ok((module, function, args)) => Ok(Ready {
                module,
                function,
                args,
            }),
            Err((module, function, args)) => Err(Promised {
                module,
                function,
                args,
            }),
        }
    }
}

impl From<Promised> for Builder {
    fn from(promised: Promised) -> Self {
        Builder {
            module: promised.module.try_resolve().ok(),
            function: promised.function.try_resolve().ok(),
            args: promised.args.try_resolve().ok(),
        }
    }
}

/// A variant meant for delegation, where fields may be omitted
pub type Builder = Generic<Option<Module>, Option<String>, Option<Vec<Ipld>>>;

impl NoParents for Builder {}

impl From<Builder> for arguments::Named<Ipld> {
    fn from(builder: Builder) -> Self {
        let mut btree = BTreeMap::new();
        if let Some(module) = builder.module {
            btree.insert("module".into(), Ipld::from(module));
        }

        if let Some(function) = builder.function {
            btree.insert("function".into(), Ipld::String(function));
        }

        if let Some(args) = builder.args {
            btree.insert("args".into(), Ipld::List(args));
        }

        arguments::Named(btree)
    }
}

impl From<Ready> for Builder {
    fn from(ready: Ready) -> Builder {
        Builder {
            module: Some(ready.module),
            function: Some(ready.function),
            args: Some(ready.args),
        }
    }
}

impl TryFrom<Builder> for Ready {
    type Error = (); // FIXME

    fn try_from(b: Builder) -> Result<Self, Self::Error> {
        Ok(Ready {
            module: b.module.ok_or(())?,
            function: b.function.ok_or(())?,
            args: b.args.ok_or(())?,
        })
    }
}

impl CheckSame for Builder {
    type Error = (); // FIXME

    fn check_same(&self, proof: &Self) -> Result<(), Self::Error> {
        if let Some(module) = &self.module {
            if module != proof.module.as_ref().unwrap() {
                return Err(());
            }
        }

        if let Some(function) = &self.function {
            if function != proof.function.as_ref().unwrap() {
                return Err(());
            }
        }

        if let Some(args) = &self.args {
            if args != proof.args.as_ref().unwrap() {
                return Err(());
            }
        }

        Ok(())
    }
}

/// A variant meant for linking together invocations with promises
pub type Promised =
    Generic<promise::Resolves<Module>, promise::Resolves<String>, promise::Resolves<Vec<Ipld>>>;

impl From<Ready> for Promised {
    fn from(ready: Ready) -> Self {
        Promised {
            module: promise::Resolves::from(Ok(ready.module)),
            function: promise::Resolves::from(Ok(ready.function)),
            args: promise::Resolves::from(Ok(ready.args)),
        }
    }
}

impl TryFrom<Promised> for Ready {
    type Error = (); // FIXME

    fn try_from(promised: Promised) -> Result<Self, Self::Error> {
        Ok(Ready {
            module: promised.module.try_resolve().map_err(|_| ())?,
            function: promised.function.try_resolve().map_err(|_| ())?,
            args: promised.args.try_resolve().map_err(|_| ())?,
        })
    }
}

impl From<Promised> for arguments::Named<Ipld> {
    fn from(promised: Promised) -> Self {
        arguments::Named::from_iter([
            ("module".into(), promised.module.into()),
            ("function".into(), promised.function.into()),
            ("args".into(), promised.args.into()),
        ])
    }
}
