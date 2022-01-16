use crate::{
    capability::{Capability, CapabilityIterator},
    ucan::Ucan,
};
use anyhow::{anyhow, Result};

pub struct ProofChain {
    ucan: Ucan,
    proofs: Vec<ProofChain>,
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

        Ok(ProofChain { ucan, proofs })
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

    /// A reduced list of capabilities that have been authoritatively
    /// delegated to the UCAN at this link in the chain, taking into
    /// account the chain of delegation through ancestral proofs
    pub fn reduce_capabilities<T: Capability>(&self) -> Vec<T> {
        let ancestral_capabilities: Vec<T> = self
            .proofs
            .iter()
            .map(|chain| chain.reduce_capabilities::<T>())
            .flatten()
            .collect();

        let iter = CapabilityIterator::<T>::new(&self.ucan);

        match self.proofs.len() {
            0 => iter.collect(),
            _ => iter
                .filter_map(|capability| {
                    for ancestral_capability in ancestral_capabilities.iter() {
                        match ancestral_capability.delegate_to(&capability) {
                            Some(delegated) => return Some(delegated),
                            None => continue,
                        };
                    }

                    None
                })
                .collect(),
        }
    }
}

impl TryFrom<Ucan> for ProofChain {
    fn try_from(ucan: Ucan) -> Result<Self> {
        ProofChain::from_ucan(ucan)
    }

    type Error = anyhow::Error;
}
