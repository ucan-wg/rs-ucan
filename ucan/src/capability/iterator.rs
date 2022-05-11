use std::marker::PhantomData;

use crate::ucan::Ucan;

use super::{Action, Capability, CapabilitySemantics, RawCapability, Scope};
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
        while let Some(capability_json) = self.ucan.attenuation().get(self.index) {
            self.index += 1;

            let (raw_with, raw_can) = match serde_json::from_value(capability_json.clone()) {
                Ok(RawCapability { with, can }) => (with, can),
                _ => continue,
            };

            match self.semantics.parse(raw_with, raw_can) {
                Some(capability) => return Some(capability),
                None => continue,
            };
        }

        None
    }
}
