use libipld_core::{ipld::Ipld, link::Link};

#[derive(Debug, Clone, PartialEq)]
pub struct Run {
    pub module: Module,
    pub function: String,
    pub args: Vec<Ipld>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Module {
    Inline(Vec<u8>),
    Cid(Link<Vec<u8>>),
}
