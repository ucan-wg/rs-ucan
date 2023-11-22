//! A builder for creating UCANs

use async_signature::AsyncSigner;
use cid::multihash;
use serde::{de::DeserializeOwned, Serialize};
use signature::Signer;

use crate::{
    capability::{Capabilities, Capability, CapabilityParser, DefaultCapabilityParser},
    crypto::{JWSSignature, SignerDid},
    error::Error,
    time,
    ucan::{Ucan, UcanHeader, UcanPayload, UCAN_VERSION},
    CidString, DefaultFact,
};

/// A builder for creating UCANs
#[derive(Debug, Clone)]
pub struct UcanBuilder<F = DefaultFact, C = DefaultCapabilityParser> {
    version: Option<String>,
    audience: Option<String>,
    nonce: Option<String>,
    capabilities: Capabilities<C>,
    lifetime: Option<u64>,
    expiration: Option<u64>,
    not_before: Option<u64>,
    facts: Option<F>,
    proofs: Option<Vec<CidString>>,
}

impl<F, C> Default for UcanBuilder<F, C> {
    fn default() -> Self {
        Self {
            version: Default::default(),
            audience: Default::default(),
            nonce: Default::default(),
            capabilities: Default::default(),
            lifetime: Default::default(),
            expiration: Default::default(),
            not_before: Default::default(),
            facts: Default::default(),
            proofs: Default::default(),
        }
    }
}

impl<F, C> UcanBuilder<F, C>
where
    F: Clone + Serialize,
    C: CapabilityParser,
{
    /// Set the UCAN version
    pub fn version(mut self, version: &str) -> Self {
        self.version = Some(version.to_string());
        self
    }

    /// Set the audience of the UCAN
    pub fn for_audience(mut self, audience: impl AsRef<str>) -> Self {
        self.audience = Some(audience.as_ref().to_string());
        self
    }

    /// Set the nonce of the UCAN
    pub fn with_nonce(mut self, nonce: impl AsRef<str>) -> Self {
        self.nonce = Some(nonce.as_ref().to_string());
        self
    }

    /// Set the lifetime of the UCAN
    pub fn with_lifetime(mut self, seconds: u64) -> Self {
        self.lifetime = Some(seconds);
        self
    }

    /// Set the expiration of the UCAN
    pub fn with_expiration(mut self, timestamp: u64) -> Self {
        self.expiration = Some(timestamp);
        self
    }

    /// Set the not before of the UCAN
    pub fn not_before(mut self, timestamp: u64) -> Self {
        self.not_before = Some(timestamp);
        self
    }

    /// Set the fact of the UCAN
    pub fn with_fact(mut self, fact: F) -> Self {
        self.facts = Some(fact);
        self
    }

    /// Add a witness to the proofs of the UCAN
    pub fn witnessed_by<F2, C2>(
        mut self,
        authority: &Ucan<F2, C2>,
        hasher: Option<multihash::Code>,
    ) -> Self
    where
        F2: Clone + DeserializeOwned,
        C2: CapabilityParser,
    {
        match authority.to_cid(hasher) {
            Ok(cid) => {
                self.proofs
                    .get_or_insert(Default::default())
                    .push(CidString(cid));
            }
            Err(e) => panic!("Failed to add authority: {}", e),
        }

        self
    }

    /// Claim a capability for the UCAN
    pub fn claiming_capability(mut self, capability: Capability) -> Self {
        self.capabilities.push(capability);
        self
    }

    /// Claim multiple capabilities for the UCAN
    pub fn claiming_capabilities(mut self, capabilities: &[Capability]) -> Self {
        self.capabilities.extend_from_slice(capabilities);
        self
    }

    /// Sign the UCAN with the given signer
    pub fn sign<S, K>(self, signer: &S) -> Result<Ucan<F, C>, Error>
    where
        S: Signer<K> + SignerDid,
        K: JWSSignature,
    {
        let version = self.version.unwrap_or_else(|| UCAN_VERSION.to_string());

        let issuer = signer.did().map_err(|e| Error::SigningError {
            msg: format!("failed to construct DID, {}", e),
        })?;

        let Some(audience) = self.audience else {
            return Err(Error::SigningError {
                msg: "an audience is required".to_string(),
            });
        };

        let header = jose_b64::serde::Json::new(UcanHeader {
            alg: K::ALGORITHM.to_string(),
            typ: "JWT".to_string(),
        })
        .map_err(|e| Error::InternalUcanError { msg: e.to_string() })?;

        let expiration = match (self.expiration, self.lifetime) {
            (None, None) => None,
            (None, Some(lifetime)) => Some(time::now() + lifetime),
            (Some(expiration), None) => Some(expiration),
            (Some(_), Some(_)) => {
                return Err(Error::SigningError {
                    msg: "only one of expiration or lifetime may be set".to_string(),
                })
            }
        };

        let payload = jose_b64::serde::Json::new(UcanPayload {
            ucv: version,
            iss: issuer,
            aud: audience,
            exp: expiration,
            nbf: self.not_before,
            nnc: self.nonce,
            cap: self.capabilities,
            fct: self.facts,
            prf: self.proofs,
        })
        .map_err(|e| Error::InternalUcanError { msg: e.to_string() })?;

        let header_b64 = serde_json::to_value(&header)
            .map_err(|e| Error::InternalUcanError { msg: e.to_string() })?;

        let payload_b64 = serde_json::to_value(&payload)
            .map_err(|e| Error::InternalUcanError { msg: e.to_string() })?;

        let signed_data = format!(
            "{}.{}",
            header_b64.as_str().ok_or(Error::InternalUcanError {
                msg: "Expected base64 encoding of header".to_string(),
            })?,
            payload_b64.as_str().ok_or(Error::InternalUcanError {
                msg: "Expected base64 encoding of payload".to_string(),
            })?,
        );

        let signature = signer.sign(signed_data.as_bytes()).to_vec().into();

        Ok(Ucan::<F, C> {
            header,
            payload,
            signature,
        })
    }

    /// Sign the UCAN with the given async signer
    pub async fn sign_async<S, K>(self, signer: &S) -> Result<Ucan<F, C>, Error>
    where
        S: AsyncSigner<K> + SignerDid,
        K: JWSSignature + 'static,
    {
        let version = self.version.unwrap_or_else(|| UCAN_VERSION.to_string());

        let issuer = signer.did().map_err(|e| Error::SigningError {
            msg: format!("failed to construct DID, {}", e),
        })?;

        let Some(audience) = self.audience else {
            return Err(Error::SigningError {
                msg: "an audience is required".to_string(),
            });
        };

        let header = jose_b64::serde::Json::new(UcanHeader {
            alg: K::ALGORITHM.to_string(),
            typ: "JWT".to_string(),
        })
        .map_err(|e| Error::InternalUcanError { msg: e.to_string() })?;

        let expiration = match (self.expiration, self.lifetime) {
            (None, None) => None,
            (None, Some(lifetime)) => Some(time::now() + lifetime),
            (Some(expiration), None) => Some(expiration),
            (Some(_), Some(_)) => {
                return Err(Error::SigningError {
                    msg: "only one of expiration or lifetime may be set".to_string(),
                })
            }
        };

        let payload = jose_b64::serde::Json::new(UcanPayload {
            ucv: version,
            iss: issuer,
            aud: audience,
            exp: expiration,
            nbf: self.not_before,
            nnc: self.nonce,
            cap: self.capabilities,
            fct: self.facts,
            prf: self.proofs,
        })
        .map_err(|e| Error::InternalUcanError { msg: e.to_string() })?;

        let header_b64 = serde_json::to_value(&header)
            .map_err(|e| Error::InternalUcanError { msg: e.to_string() })?;

        let payload_b64 = serde_json::to_value(&payload)
            .map_err(|e| Error::InternalUcanError { msg: e.to_string() })?;

        let signed_data = format!(
            "{}.{}",
            header_b64.as_str().ok_or(Error::InternalUcanError {
                msg: "Expected base64 encoding of header".to_string(),
            })?,
            payload_b64.as_str().ok_or(Error::InternalUcanError {
                msg: "Expected base64 encoding of payload".to_string(),
            })?,
        );

        let signature = signer
            .sign_async(signed_data.as_bytes())
            .await
            .map_err(|e| Error::SigningError { msg: e.to_string() })?
            .to_vec()
            .into();

        Ok(Ucan::<F, C> {
            header,
            payload,
            signature,
        })
    }
}

#[cfg(test)]
mod tests {
    use signature::rand_core;
    use std::str::FromStr;

    use crate::did_verifier::DidVerifierMap;

    use super::*;

    #[test]
    fn test_round_trip_validate() -> Result<(), anyhow::Error> {
        let did_verifier_map = DidVerifierMap::default();

        let iss_key = ed25519_dalek::SigningKey::generate(&mut rand_core::OsRng);
        let aud_key = ed25519_dalek::SigningKey::generate(&mut rand_core::OsRng);

        let ucan: Ucan = UcanBuilder::default()
            .for_audience(aud_key.did()?)
            .sign(&iss_key)?;

        let token = ucan.encode()?;
        let decoded: Ucan = Ucan::from_str(&token)?;

        assert!(decoded.validate(0, &did_verifier_map).is_ok());

        Ok(())
    }
}
