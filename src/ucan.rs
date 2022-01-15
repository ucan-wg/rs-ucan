use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};
use std::str;

use crate::crypto::verify_signature;
use crate::time::now;
use crate::types::{Capability, Fact};

#[derive(Serialize, Deserialize, Debug)]
pub struct UcanHeader {
    pub alg: String,
    pub typ: String,
    pub ucv: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UcanPayload {
    pub iss: String,
    pub aud: String,
    pub exp: u64,
    pub nbf: Option<u64>,
    pub nnc: Option<String>,
    pub att: Vec<Capability>,
    pub fct: Vec<Fact>,
    pub prf: Vec<String>,
}

pub struct Ucan {
    header: UcanHeader,
    payload: UcanPayload,
    signed_data: Vec<u8>,
    signature: Vec<u8>,
}

impl Ucan {
    pub fn new(
        header: UcanHeader,
        payload: UcanPayload,
        signed_data: Vec<u8>,
        signature: Vec<u8>,
    ) -> Self {
        Ucan {
            signed_data,
            header,
            payload,
            signature,
        }
    }

    /**
     * Deserialize an encoded UCAN token string into a UCAN
     */
    pub fn from_token_string(ucan_token_string: &str) -> Result<Ucan> {
        let mut parts = ucan_token_string
            .split('.')
            .map(|str| base64::decode(str).map_err(|error| anyhow!(error)));

        let mut signed_data: Vec<u8> = Vec::new();

        let header: UcanHeader = match parts.next() {
            Some(part) => match part {
                Ok(mut decoded) => {
                    let header: UcanHeader = match serde_json::from_slice(decoded.as_slice()) {
                        Ok(header) => header,
                        Err(error) => {
                            return Err(error).context("Could not parse UCAN header JSON")
                        }
                    };

                    signed_data.append(&mut decoded);

                    header
                }
                Err(error) => return Err(error).context("Could not decode UCAN header base64"),
            },
            None => return Err(anyhow!("Missing UCAN header in token part")),
        };

        signed_data.extend_from_slice(".".as_bytes());

        let payload: UcanPayload = match parts.next() {
            Some(part) => match part {
                Ok(mut decoded) => {
                    let payload: UcanPayload = match serde_json::from_slice(decoded.as_slice()) {
                        Ok(payload) => payload,
                        Err(error) => {
                            return Err(error).context("Could not parse UCAN payload JSON")
                        }
                    };

                    signed_data.append(&mut decoded);

                    payload
                }
                Err(error) => return Err(error).context("Could not parse UCAN payload base64"),
            },
            None => return Err(anyhow!("Missing UCAN payload in token part")),
        };

        let signature: Vec<u8> = match parts.next() {
            Some(part) => match part {
                Ok(decoded) => decoded,
                Err(error) => return Err(error).context("Could not parse UCAN signature base64"),
            },
            None => return Err(anyhow!("Missing UCAN signature in token part")),
        };

        Ok(Ucan::new(header, payload, signed_data, signature))
    }

    /**
     * Validate the UCAN's signature and timestamps
     */
    pub fn validate(&self) -> Result<()> {
        if self.is_expired() {
            return Err(anyhow!("Expired"));
        }

        if self.is_too_early() {
            return Err(anyhow!("Not active yet (too early)"));
        }

        self.check_signature()
    }

    fn check_signature(&self) -> Result<()> {
        match verify_signature(&self.signed_data, &self.signature, &self.payload.iss) {
            Err(error) => Err(error).context(format!(
                "Invalid signature on UCAN ({:?}, {:?}, signature: {:?})",
                self.header, self.payload, self.signature
            )),
            _ => Ok(()),
        }
    }

    /**
     * Returns true if the UCAN has past its expiration date
     */
    fn is_expired(&self) -> bool {
        self.payload.exp > now()
    }

    /**
     * Returns true if the not-before ("nbf") time is still in the future
     */
    fn is_too_early(&self) -> bool {
        match self.payload.nbf {
            Some(nbf) => nbf > now(),
            None => false,
        }
    }

    pub fn algorithm(&self) -> &String {
        &self.header.alg
    }

    pub fn issuer(&self) -> &String {
        &self.payload.iss
    }

    pub fn audience(&self) -> &String {
        &self.payload.aud
    }

    pub fn proofs(&self) -> &Vec<String> {
        &self.payload.prf
    }

    pub fn expires_at(&self) -> &u64 {
        &self.payload.exp
    }

    pub fn not_before(&self) -> &Option<u64> {
        &self.payload.nbf
    }

    pub fn nonce(&self) -> &Option<String> {
        &self.payload.nnc
    }

    pub fn attenuation(&self) -> &Vec<Capability> {
        &self.payload.att
    }

    pub fn facts(&self) -> &Vec<Fact> {
        &self.payload.fct
    }
}
