use async_recursion::async_recursion;
use std::collections::BTreeSet;
use std::fmt::Debug;

use crate::{
    capability::{
        proof::{ProofDelegationSemantics, ProofSelection},
        Action, Capability, CapabilityIterator, CapabilitySemantics, Resource, Scope, With,
    },
    crypto::did::DidParser,
    ucan::Ucan,
};
use anyhow::{anyhow, Result};

const PROOF_DELEGATION_SEMANTICS: ProofDelegationSemantics = ProofDelegationSemantics {};

#[derive(PartialEq)]
pub struct CapabilityInfo<S: Scope, A: Action> {
    pub originators: BTreeSet<String>,
    pub not_before: Option<u64>,
    pub expires_at: u64,
    pub capability: Capability<S, A>,
}

impl<S, A> Debug for CapabilityInfo<S, A>
where
    S: Scope,
    A: Action,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CapabilityInfo")
            .field("originators", &self.originators)
            .field("not_before", &self.not_before)
            .field("expires_at", &self.expires_at)
            .field("capability", &self.capability)
            .finish()
    }
}

/// A deserialized chain of ancestral proofs that are linked to a UCAN
pub struct ProofChain {
    ucan: Ucan,
    proofs: Vec<ProofChain>,
    redelegations: BTreeSet<usize>,
}

impl ProofChain {
    #[async_recursion(?Send)]
    pub async fn from_ucan<'a>(ucan: Ucan, did_parser: &'a DidParser) -> Result<ProofChain> {
        ucan.validate(did_parser).await?;

        let mut proofs: Vec<ProofChain> = Vec::new();

        for proof_string in ucan.proofs().iter() {
            let proof_chain = Self::try_from_token_string(proof_string, did_parser).await?;
            proof_chain.validate_link_to(&ucan)?;
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

    pub async fn from_cid<'a>(_cid: &str, _did_parser: &'a DidParser) -> Result<ProofChain> {
        todo!("Resolving a proof from a CID not yet implemented")
    }

    pub async fn try_from_token_string<'a>(
        ucan_token_string: &str,
        did_parser: &'a DidParser,
    ) -> Result<ProofChain> {
        let ucan = Ucan::try_from_token_string(ucan_token_string)?;
        Self::from_ucan(ucan, did_parser).await
    }

    fn validate_link_to(&self, ucan: &Ucan) -> Result<()> {
        let audience = self.ucan.audience();
        let issuer = ucan.issuer();

        match audience == issuer {
            true => match self.ucan.lifetime_encompasses(&ucan) {
                true => Ok(()),
                false => Err(anyhow!("Invalid UCAN link: lifetime exceeds attenuation")),
            },
            false => Err(anyhow!(
                "Invalid UCAN link: audience {} does not match issuer {}",
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
    ) -> Vec<CapabilityInfo<S, A>>
    where
        Semantics: CapabilitySemantics<S, A>,
        S: Scope,
        A: Action,
    {
        // Get the set of inherited attenuations (excluding redelegations)
        // before further attenuating by own lifetime and capabilities:
        let ancestral_capability_infos: Vec<CapabilityInfo<S, A>> = self
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

        // Get the set of capabilities that are blanket redelegated from
        // ancestor proofs (via the prf: resource):
        let mut redelegated_capability_infos: Vec<CapabilityInfo<S, A>> = self
            .redelegations
            .iter()
            .map(|index| {
                self.proofs
                    .get(*index)
                    .unwrap()
                    .reduce_capabilities(semantics)
                    .into_iter()
                    .map(|mut info| {
                        // Redelegated capabilities should be attenuated by
                        // this UCAN's lifetime
                        info.not_before = self.ucan.not_before().clone();
                        info.expires_at = self.ucan.expires_at().clone();
                        info
                    })
            })
            .flatten()
            .collect();

        let self_capabilities_iter = CapabilityIterator::new(&self.ucan, semantics);

        // Get the claimed attenuations of this ucan, cross-checking ancestral
        // attenuations to discover the originating authority
        let mut self_capability_infos: Vec<CapabilityInfo<S, A>> = match self.proofs.len() {
            0 => self_capabilities_iter
                .map(|capability| CapabilityInfo {
                    originators: BTreeSet::from_iter(vec![self.ucan.issuer().clone()]),
                    capability,
                    not_before: self.ucan.not_before().clone(),
                    expires_at: self.ucan.expires_at().clone(),
                })
                .collect(),
            _ => self_capabilities_iter
                .filter_map(|capability| {
                    let mut originators = BTreeSet::<String>::new();

                    for ancestral_capability_info in ancestral_capability_infos.iter() {
                        match ancestral_capability_info.capability.enables(&capability) {
                            true => {
                                originators.extend(ancestral_capability_info.originators.clone())
                            }
                            // true => return Some(capability),
                            false => continue,
                        }
                    }

                    // If there are no related ancestral capability, then this
                    // link in the chain is considered the first originator
                    if originators.len() == 0 {
                        originators.insert(self.ucan.issuer().clone());
                    }

                    Some(CapabilityInfo {
                        capability,
                        originators,
                        not_before: self.ucan.not_before().clone(),
                        expires_at: self.ucan.expires_at().clone(),
                    })
                })
                .collect(),
        };

        self_capability_infos.append(&mut redelegated_capability_infos);

        let mut merged_capability_infos = Vec::<CapabilityInfo<S, A>>::new();

        // Merge redundant capabilities (accounting for redelegation), ensuring
        // that discrete originators are aggregated as we go
        'merge: while let Some(capability_info) = self_capability_infos.pop() {
            for remaining_capability_info in &mut self_capability_infos {
                if remaining_capability_info
                    .capability
                    .enables(&capability_info.capability)
                {
                    remaining_capability_info
                        .originators
                        .extend(capability_info.originators);
                    continue 'merge;
                }
            }

            merged_capability_infos.push(capability_info);
        }

        merged_capability_infos
    }
}

// impl TryFrom<Ucan> for ProofChain {
//     fn try_from(ucan: Ucan) -> Result<Self> {
//         ProofChain::from_ucan(ucan)
//     }

//     type Error = anyhow::Error;
// }
