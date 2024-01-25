use libipld_core::ipld::Ipld;

pub mod common;

pub trait Condition {
    fn validate(&self, ipld: &Ipld) -> bool;
}
