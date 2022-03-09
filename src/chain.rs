use std::collections::BTreeSet;

use crate::{
    capability::{
        proof::{ProofDelegationSemantics, ProofSelection},
        Action, Capability, CapabilityIterator, CapabilitySemantics, Resource, Scope, With,
    },
    ucan::Ucan,
};
use anyhow::{anyhow, Result};

const PROOF_DELEGATION_SEMANTICS: ProofDelegationSemantics = ProofDelegationSemantics {};

/// A deserialized chain of ancestral proofs that are linked to a UCAN
pub struct ProofChain {
    ucan: Ucan,
    proofs: Vec<ProofChain>,
    redelegations: BTreeSet<usize>,
}

impl ProofChain {
    pub fn from_ucan(ucan: Ucan) -> Result<ProofChain> {
        ucan.validate()?;

        let mut proofs: Vec<ProofChain> = Vec::new();

        for proof_string in ucan.proofs().iter() {
            let proof_chain = Self::from_token_string(proof_string)?;
            proof_chain.check_link_to(&ucan)?;
            proofs.push(proof_chain);
        }

        let mut redelegations = BTreeSet::<usize>::new();

        for capability in CapabilityIterator::new(&ucan, &PROOF_DELEGATION_SEMANTICS) {
            match capability.with() {
                With::Resource {
                    kind: Resource::Scoped(ProofSelection::All),
                } => {
                    for index in 0..proofs.len() {
                        redelegations.insert(index);
                    }
                }
                With::Resource {
                    kind: Resource::Scoped(ProofSelection::Index(index)),
                } => {
                    if *index < proofs.len() {
                        redelegations.insert(index.clone());
                    } else {
                        return Err(anyhow!(
                            "Unable to redelegate proof; no proof at zero-based index {}",
                            index
                        ));
                    }
                }
                _ => continue,
            }
        }

        Ok(ProofChain {
            ucan,
            proofs,
            redelegations,
        })
    }

    pub async fn from_cid(_cid: &str) -> Result<ProofChain> {
        todo!("Resolving a proof from a CID not yet implemented")
    }

    pub fn from_token_string(ucan_token_string: &str) -> Result<ProofChain> {
        let ucan = Ucan::from_token_string(ucan_token_string)?;
        Self::from_ucan(ucan)
    }

    fn check_link_to(&self, ucan: &Ucan) -> Result<()> {
        let audience = self.ucan.audience();
        let issuer = ucan.issuer();

        match audience == issuer {
            true => Ok(()),
            false => Err(anyhow!(
                "Invalid UCAN. Audience {} does not match issuer {}",
                audience,
                issuer
            )),
        }
    }

    pub fn ucan(&self) -> &Ucan {
        &self.ucan
    }

    pub fn proofs(&self) -> &Vec<ProofChain> {
        &self.proofs
    }

    pub fn reduce_capabilities<Semantics, S, A>(
        &self,
        semantics: &Semantics,
    ) -> Vec<Capability<S, A>>
    where
        Semantics: CapabilitySemantics<S, A>,
        S: Scope,
        A: Action,
    {
        let ancestral_capabilities: Vec<Capability<S, A>> = self
            .proofs
            .iter()
            .enumerate()
            .map(|(index, ancestor_chain)| {
                if self.redelegations.contains(&index) {
                    Vec::new()
                } else {
                    ancestor_chain.reduce_capabilities(semantics)
                }
            })
            .flatten()
            .collect();

        let mut redelegated_capabilities: Vec<Capability<S, A>> = self
            .redelegations
            .iter()
            .map(|index| {
                self.proofs
                    .get(*index)
                    .unwrap()
                    .reduce_capabilities(semantics)
            })
            .flatten()
            .collect();

        let self_capabilities_iter = CapabilityIterator::new(&self.ucan, semantics);

        let mut self_capabilities: Vec<Capability<S, A>> = match self.proofs.len() {
            0 => self_capabilities_iter.collect(),
            _ => self_capabilities_iter
                .filter_map(|capability| {
                    for ancestral_capability in ancestral_capabilities.iter() {
                        match ancestral_capability.enables(&capability) {
                            true => return Some(capability),
                            false => continue,
                        }
                    }

                    // TODO: No ancestor witnesses this capability. Right
                    // now we just discard the capability. Should this be a
                    // critical failure?
                    None
                })
                .collect(),
        };

        self_capabilities.append(&mut redelegated_capabilities);

        let mut merged_capabilities = Vec::<Capability<S, A>>::new();

        'merge: while let Some(capability) = self_capabilities.pop() {
            for remaining_capability in &self_capabilities {
                if remaining_capability.enables(&capability) {
                    continue 'merge;
                }
            }

            merged_capabilities.push(capability);
        }

        merged_capabilities
    }
}

impl TryFrom<Ucan> for ProofChain {
    fn try_from(ucan: Ucan) -> Result<Self> {
        ProofChain::from_ucan(ucan)
    }

    type Error = anyhow::Error;
}
