//! Ability to run a Wasm module

use super::module::Module;
use crate::{
    ability::{arguments, command::Command},
    // delegation::Delegable,
    invocation::promise,
    ipld,
    // proof::{parentless::NoParents, same::CheckSame},
};
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};

const COMMAND: &'static str = "/wasm/run";

impl Command for Run {
    const COMMAND: &'static str = COMMAND;
}

// FIXME autogenerate for resolvable?
impl Command for PromisedRun {
    const COMMAND: &'static str = COMMAND;
}

/// The ability to run a Wasm module on the subject's machine
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Run {
    /// The Wasm module to run
    pub module: Module,

    /// The function from the module to run
    pub function: String,

    /// Arguments to pass to the function
    pub args: Vec<Ipld>,
}

impl TryFrom<arguments::Named<Ipld>> for Run {
    type Error = ();

    fn try_from(named: arguments::Named<Ipld>) -> Result<Self, Self::Error> {
        let mut module = None;
        let mut function = None;
        let mut args = None;

        for (key, ipld) in named {
            match key.as_str() {
                "mod" => {
                    module = Some(ipld.try_into().map_err(|_| ())?);
                }
                "fun" => {
                    if let Ipld::String(s) = ipld {
                        function = Some(s);
                    } else {
                        return Err(());
                    }
                }
                "args" => {
                    if let Ipld::List(list) = ipld {
                        args = Some(list);
                    } else {
                        return Err(());
                    }
                }
                _ => return Err(()),
            }
        }

        Ok(Run {
            module: module.ok_or(())?,
            function: function.ok_or(())?,
            args: args.ok_or(())?,
        })
    }
}

impl promise::Resolvable for Run {
    type Promised = PromisedRun;
}

/// A variant meant for linking together invocations with promises
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PromisedRun {
    pub module: promise::Resolves<Module>,
    pub function: promise::Resolves<String>,
    pub args: promise::Resolves<Vec<ipld::Promised>>,
}

impl TryFrom<arguments::Named<ipld::Promised>> for PromisedRun {
    type Error = ();

    fn try_from(named: arguments::Named<ipld::Promised>) -> Result<Self, Self::Error> {
        let mut module = None;
        let mut function = None;
        let mut args = None;

        for (key, prom) in named {
            match key.as_str() {
                "module" => module = Some(prom.try_into().map_err(|_| ())?),
                "function" => function = Some(prom.try_into().map_err(|_| ())?),
                "args" => {
                    if let ipld::Promised::List(list) = prom.into() {
                        args = Some(promise::Resolves::new(list));
                    } else {
                        return Err(());
                    }
                }
                _ => return Err(()),
            }
        }

        Ok(PromisedRun {
            module: module.ok_or(())?,
            function: function.ok_or(())?,
            args: args.ok_or(())?,
        })
    }
}

impl From<Run> for PromisedRun {
    fn from(ready: Run) -> Self {
        PromisedRun {
            module: promise::Resolves::from(Ok(ready.module)),
            function: promise::Resolves::from(Ok(ready.function)),
            args: promise::Resolves::from(Ok(ready
                .args
                .iter()
                .map(|ipld| ipld.clone().into())
                .collect())),
        }
    }
}

impl From<PromisedRun> for arguments::Named<ipld::Promised> {
    fn from(promised: PromisedRun) -> Self {
        arguments::Named::from_iter([
            ("module".into(), promised.module.into()),
            ("function".into(), promised.function.into()),
            ("args".into(), promised.args.into()),
        ])
    }
}
