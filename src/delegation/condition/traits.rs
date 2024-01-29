use libipld_core::ipld::Ipld;

pub trait Condition {
    fn validate(&self, ipld: &Ipld) -> bool;
}
