//! Traits for abstracting over conditions.

use crate::ability::arguments;
use libipld_core::ipld::Ipld;

/// A trait for conditions that can be run on named IPLD arguments.
pub trait Condition {
    /// Check that some condition is met on named IPLD arguments.
    fn validate(&self, args: &arguments::Named<Ipld>) -> bool;
}
