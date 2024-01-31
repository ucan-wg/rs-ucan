use super::command::Command;
use crate::proof::{parentless::NoParents, same::CheckSame};
use libipld_core::{ipld::Ipld, link::Link};

#[derive(Debug, Clone, PartialEq)]
pub struct Run {
    pub module: Module,
    pub function: String,
    pub args: Vec<Ipld>,
}

// FIXME
#[derive(Debug, Clone, PartialEq)]
pub enum Module {
    Inline(Vec<u8>),
    Cid(Link<Vec<u8>>),
}

impl Command for Run {
    const COMMAND: &'static str = "wasm/run";
}

impl NoParents for Run {}

impl CheckSame for Run {
    type Error = (); // FIXME
    fn check_same(&self, _proof: &Self) -> Result<(), Self::Error> {
        Ok(()) // FIXME
    }
}
