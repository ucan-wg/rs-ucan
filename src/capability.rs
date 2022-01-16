use std::marker::PhantomData;

use serde::{de::DeserializeOwned, Serialize};

use crate::Ucan;

/// A trait that must be implemented by capabilities
pub trait Capability: Serialize + DeserializeOwned {
    /// Delegate this capability to an implied descendant, returning an
    /// appropriately attenuated capability, or None if the delegation cannot
    /// take place
    fn delegate_to(&self, _other: &Self) -> Option<Self> {
        None
    }
}

/// An iterator over all capabilities in a UCAN's attenuations field that
/// deserialize to a specified type
pub struct CapabilityIterator<'a, T>
where
    T: Capability,
{
    index: usize,
    ucan: &'a Ucan,
    capability_type: PhantomData<T>,
}

impl<'a, T> CapabilityIterator<'a, T>
where
    T: Capability,
{
    pub fn new(ucan: &'a Ucan) -> Self {
        CapabilityIterator {
            index: 0,
            ucan,
            capability_type: PhantomData::<T>,
        }
    }
}

impl<'a, T> Iterator for CapabilityIterator<'a, T>
where
    T: Capability,
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        while let Some(raw_capability) = self.ucan.attenuation().get(self.index) {
            self.index = self.index + 1;
            match serde_json::from_value(raw_capability.clone()) {
                Ok(value) => return Some(value),
                _ => continue,
            }
        }

        None
    }
}
