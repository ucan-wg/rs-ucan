use crate::chain::ProofChain;

// Placeholder types:

pub type CapabilitySemantics = ();

pub type UcanStore = ();

pub enum CapabilityAuthority {
    Proof(ProofChain),
    Store(UcanStore),
}
