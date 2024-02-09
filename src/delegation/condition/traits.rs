use crate::ability::arguments;
use libipld_core::ipld::Ipld;

pub trait Condition: TryFrom<Ipld> + Into<Ipld> {
    fn validate(&self, args: &arguments::Named<Ipld>) -> bool;
}
