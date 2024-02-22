use libipld_core::cid::Cid;

// AKA Selector
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Pending {
    Ok(Cid),
    Err(Cid),
    Any(Cid),
}
