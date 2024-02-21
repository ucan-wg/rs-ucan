//! Ability to run a Wasm module

use super::module::Module;
use crate::{
    ability::{
        arguments,
        command::{Command, ParseAbility},
    },
    delegation::Delegable,
    invocation::promise,
    ipld,
    proof::{parentless::NoParents, same::CheckSame},
};
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

const COMMAND: &'static str = "wasm/run";

impl Command for Ready {
    const COMMAND: &'static str = COMMAND;
}

impl Command for Builder {
    const COMMAND: &'static str = COMMAND;
}

impl Command for Promised {
    const COMMAND: &'static str = COMMAND;
}

/// The ability to run a Wasm module on the subject's machine
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Ready {
    /// The Wasm module to run
    pub module: Module,

    /// The function from the module to run
    pub function: String,

    /// Arguments to pass to the function
    pub args: Vec<Ipld>,
}

impl Delegable for Ready {
    type Builder = Builder;
}

impl TryFrom<arguments::Named<Ipld>> for Builder {
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

        Ok(Builder {
            module,
            function,
            args,
        })
    }
}

// impl TryFrom<arguments::Named<Ipld>> for Builder {
//     type Error = ();
//
//     fn try_from(args: arguments::Named<Ipld>) -> Result<Self, Self::Error> {
//         let ready = Ready::try_from(args)?;
//
//         Ok(Builder {
//             module: Some(ready.module),
//             function: Some(ready.function),
//             args: Some(ready.args),
//         })
//     }
// }

impl promise::Resolvable for Ready {
    type Promised = Promised;
}

/// A variant meant for delegation, where fields may be omitted
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Builder {
    /// The Wasm module to run
    pub module: Option<Module>,

    /// The function from the module to run
    pub function: Option<String>,

    /// Arguments to pass to the function
    pub args: Option<Vec<Ipld>>,
}

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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Promised {
    pub module: promise::Resolves<Module>,
    pub function: promise::Resolves<String>,
    pub args: promise::Resolves<Vec<ipld::Promised>>,
}

// impl From<Ready> for Promised {
//     fn from(ready: Ready) -> Self {
//         Promised {
//             module: promise::Resolves::from(Ok(ready.module)),
//             function: promise::Resolves::from(Ok(ready.function)),
//             args: promise::Resolves::from(Ok(ready.args)),
//         }
//     }
// }

// impl TryFrom<Promised> for Ready {
//     type Error = (); // FIXME
//
//     fn try_from(promised: Promised) -> Result<Self, Self::Error> {
//         Ok(Ready {
//             module: promised.module.try_from().map_err(|_| ())?,
//             function: promised.function.try_from().map_err(|_| ())?,
//             args: promised.args.try_from().map_err(|_| ())?,
//         })
//     }
// }

impl From<Promised> for arguments::Named<ipld::Promised> {
    fn from(promised: Promised) -> Self {
        arguments::Named::from_iter([
            ("module".into(), promised.module.into()),
            ("function".into(), promised.function.into()),
            ("args".into(), promised.args.into()),
        ])
    }
}
