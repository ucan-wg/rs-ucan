//! A builder for creating UCANs

use cid::multihash;
use serde::Serialize;
use signature::Signer;

use crate::{
    capability::{Capabilities, Capability, CapabilityParser, DefaultCapabilityParser},
    crypto::JWSSignature,
    error::Error,
    semantics::fact::DefaultFact,
    ucan::{Ucan, UcanHeader, UcanPayload, UCAN_VERSION},
};

/// The default multihash algorithm used for UCANs
pub const DEFAULT_MULTIHASH: multihash::Code = multihash::Code::Sha2_256;

/// A builder for creating UCANs
#[derive(Debug, Clone)]
pub struct UcanBuilder<F = DefaultFact, C = DefaultCapabilityParser> {
    version: Option<String>,
    issuer: Option<String>,
    audience: Option<String>,
    nonce: Option<String>,
    capabilities: Capabilities<C>,
    lifetime: Option<u64>,
    expiration: Option<u64>,
    not_before: Option<u64>,
    facts: Option<F>,
    proofs: Option<Vec<String>>,
}

impl<F, C> Default for UcanBuilder<F, C> {
    fn default() -> Self {
        Self {
            version: Default::default(),
            issuer: Default::default(),
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
    F: Serialize,
    C: CapabilityParser,
{
    /// Set the UCAN version
    pub fn version(mut self, version: &str) -> Self {
        self.version = Some(version.to_string());
        self
    }

    /// Set the issuer of the UCAN
    pub fn issued_by<S: AsRef<str>>(mut self, issuer: S) -> Self {
        self.issuer = Some(issuer.as_ref().to_string());
        self
    }

    /// Set the audience of the UCAN
    pub fn for_audience<S: AsRef<str>>(mut self, audience: S) -> Self {
        self.audience = Some(audience.as_ref().to_string());
        self
    }

    /// Set the nonce of the UCAN
    pub fn with_nonce<S: AsRef<str>>(mut self, nonce: S) -> Self {
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
        F2: Serialize,
        C2: CapabilityParser,
    {
        let hasher = hasher.unwrap_or(DEFAULT_MULTIHASH);

        match authority.to_cid(hasher) {
            Ok(cid) => {
                self.proofs
                    .get_or_insert(Default::default())
                    .push(cid.to_string());
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
        S: Signer<K>,
        K: JWSSignature,
    {
        let version = self.version.unwrap_or_else(|| UCAN_VERSION.to_string());

        let Some(issuer) = self.issuer else {
            return Err(Error::SigningError {
                msg: "an issuer is required".to_string(),
            });
        };

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

        let payload = jose_b64::serde::Json::new(UcanPayload {
            ucv: version,
            iss: issuer,
            aud: audience,
            exp: self.expiration,
            nbf: self.not_before,
            nnc: self.nonce,
            cap: self.capabilities,
            fct: self.facts,
            prf: self.proofs,
        })
        .map_err(|e| Error::InternalUcanError { msg: e.to_string() })?;

        let signature = signer
            .sign(&[header.as_ref(), ".".as_bytes(), payload.as_ref()].concat())
            .to_vec()
            .into();

        Ok(Ucan {
            header,
            payload,
            signature,
        })
    }
}
