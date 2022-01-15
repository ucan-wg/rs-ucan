use crate::ucan::Ucan;
use anyhow::{anyhow, Result};

pub struct ProofChain {
    token_string: String,
    ucan: Ucan,
    proofs: Vec<ProofChain>,
}

impl ProofChain {
    pub fn from_token_string(ucan_token_string: &str) -> Result<ProofChain> {
        let ucan = Ucan::from_token_string(ucan_token_string)?;

        ucan.validate()?;

        let mut proofs: Vec<ProofChain> = Vec::new();

        for proof_string in ucan.proofs().iter() {
            let proof_chain = ProofChain::from_token_string(proof_string)?;
            proof_chain.check_link_to(&ucan)?;
            proofs.push(proof_chain);
        }

        Ok(ProofChain {
            token_string: String::from(ucan_token_string),
            ucan,
            proofs,
        })
    }

    pub fn check_link_to(&self, ucan: &Ucan) -> Result<()> {
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
}
