use libipld_core::ipld::Ipld;

pub trait Condition: TryFrom<Ipld> + Into<Ipld> {
    fn validate(&self, ipld: &Ipld) -> bool;
}
