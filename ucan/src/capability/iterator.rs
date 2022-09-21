use std::marker::PhantomData;

use crate::ucan::Ucan;

use super::{Action, Capability, CapabilityIpld, CapabilitySemantics, Scope};
pub struct CapabilityIterator<'a, Semantics, S, A>
where
    Semantics: CapabilitySemantics<S, A>,
    S: Scope,
    A: Action,
{
    index: usize,
    ucan: &'a Ucan,
    semantics: &'a Semantics,
    capability_type: PhantomData<Capability<S, A>>,
}

impl<'a, Semantics, S, A> CapabilityIterator<'a, Semantics, S, A>
where
    Semantics: CapabilitySemantics<S, A>,
    S: Scope,
    A: Action,
{
    pub fn new(ucan: &'a Ucan, semantics: &'a Semantics) -> Self {
        CapabilityIterator {
            index: 0,
            ucan,
            semantics,
            capability_type: PhantomData::<Capability<S, A>>,
        }
    }
}

impl<'a, Semantics, S, A> Iterator for CapabilityIterator<'a, Semantics, S, A>
where
    Semantics: CapabilitySemantics<S, A>,
    S: Scope,
    A: Action,
{
    type Item = Capability<S, A>;

    fn next(&mut self) -> Option<Self::Item> {
        // TODO(#22): Full support for 0.9 and the nb field
        while let Some(CapabilityIpld { with, can, .. }) = self.ucan.attenuation().get(self.index) {
            self.index += 1;

            match self.semantics.parse(with.as_str(), can.as_str()) {
                Some(capability) => return Some(capability),
                None => continue,
            };
        }

        None
    }
}
