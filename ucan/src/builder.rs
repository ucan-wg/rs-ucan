use crate::{
    capability::{
        proof::ProofDelegationSemantics, Action, Capability, CapabilitySemantics, RawCapability,
        Scope,
    },
    crypto::KeyMaterial,
    time::now,
    ucan::{UcanHeader, UcanPayload},
};
use anyhow::{anyhow, Context, Result};
use log::warn;
use serde::{de::DeserializeOwned, Serialize};
use serde_json::Value;
use textnonce::TextNonce;

use crate::ucan::Ucan;

/// A signable is a UCAN that has all the state it needs in order to be signed,
/// but has not yet been signed.
/// NOTE: This may be useful for bespoke signing flows down the road. It is
/// meant to approximate the way that ts-ucan produces an unsigned intermediate
/// artifact (e.g., <https://github.com/ucan-wg/ts-ucan/blob/e10bdeca26e663df72e4266ccd9d47f8ce100665/src/builder.ts#L257-L278>)

#[derive(Clone)]
pub struct Signable<K>
where
    K: KeyMaterial + Clone,
{
    pub issuer: K,
    pub audience: String,

    pub capabilities: Vec<Value>,

    pub expiration: u64,
    pub not_before: Option<u64>,

    pub facts: Vec<Value>,
    pub proofs: Vec<String>,
    pub add_nonce: bool,
}

impl<K> Signable<K>
where
    K: KeyMaterial + Clone,
{
    pub const UCAN_VERSION: &'static str = "0.8.1";

    /// The header field components of the UCAN JWT
    pub fn ucan_header(&self) -> UcanHeader {
        UcanHeader {
            alg: self.issuer.get_jwt_algorithm_name(),
            typ: "JWT".into(),
            ucv: Self::UCAN_VERSION.into(),
        }
    }

    /// The payload field components of the UCAN JWT
    pub async fn ucan_payload(&self) -> Result<UcanPayload> {
        let nonce = match self.add_nonce {
            true => Some(TextNonce::new().to_string()),
            false => None,
        };

        Ok(UcanPayload {
            aud: self.audience.clone(),
            iss: self.issuer.get_did().await?,
            exp: self.expiration,
            nbf: self.not_before,
            nnc: nonce,
            att: self.capabilities.clone(),
            fct: self.facts.clone(),
            prf: self.proofs.clone(),
        })
    }

    /// Produces a Ucan, which contains finalized UCAN fields along with signed
    /// data suitable for encoding as a JWT token string
    pub async fn sign(&self) -> Result<Ucan> {
        let header = self.ucan_header();
        let payload = self
            .ucan_payload()
            .await
            .expect("Unable to generate UCAN payload");

        let header_base64 = match serde_json::to_string(&header) {
            Ok(json) => base64::encode_config(json.as_bytes(), base64::URL_SAFE_NO_PAD),
            Err(error) => return Err(error).context("Unable to serialize UCAN header as JSON"),
        };

        let payload_base64 = match serde_json::to_string(&payload) {
            Ok(json) => base64::encode_config(json.as_bytes(), base64::URL_SAFE_NO_PAD),
            Err(error) => return Err(error).context("Unable to serialize UCAN payload as JSON"),
        };

        let data_to_sign = Vec::from(format!("{}.{}", header_base64, payload_base64).as_bytes());
        let signature = self.issuer.sign(data_to_sign.as_slice()).await?;

        Ok(Ucan::new(header, payload, data_to_sign, signature))
    }
}

/// A builder API for UCAN tokens
#[derive(Clone)]
pub struct UcanBuilder<K>
where
    K: KeyMaterial,
{
    issuer: Option<K>,
    audience: Option<String>,

    capabilities: Vec<Value>,

    lifetime: Option<u64>,
    expiration: Option<u64>,
    not_before: Option<u64>,

    facts: Vec<Value>,
    proofs: Vec<String>,
    add_nonce: bool,
}

impl<K> Default for UcanBuilder<K>
where
    K: KeyMaterial,
{
    /// Create an empty builder.
    /// Before finalising the builder, you need to at least call:
    ///
    /// - `issued_by`
    /// - `to_audience` and one of
    /// - `with_lifetime` or `with_expiration`.
    ///
    /// To finalise the builder, call its `build` or `build_parts` method.
    fn default() -> Self {
        UcanBuilder {
            issuer: None,
            audience: None,

            capabilities: Vec::new(),

            lifetime: None,
            expiration: None,
            not_before: None,

            facts: Vec::new(),
            proofs: Vec::new(),
            add_nonce: false,
        }
    }
}

impl<K> UcanBuilder<K>
where
    K: KeyMaterial + Clone,
{
    /// The UCAN must be signed with the private key of the issuer to be valid.
    pub fn issued_by(mut self, issuer: &K) -> Self {
        self.issuer = Some(issuer.clone());
        self
    }

    /// This is the identity this UCAN transfers rights to.
    ///
    /// It could e.g. be the DID of a service you're posting this UCAN as a JWT to,
    /// or it could be the DID of something that'll use this UCAN as a proof to
    /// continue the UCAN chain as an issuer.
    pub fn for_audience(mut self, audience: &str) -> Self {
        self.audience = Some(String::from(audience));
        self
    }

    /// The number of seconds into the future (relative to when build() is
    /// invoked) to set the expiration. This is ignored if an explicit expiration
    /// is set.
    pub fn with_lifetime(mut self, seconds: u64) -> Self {
        self.lifetime = Some(seconds);
        self
    }

    /// Set the POSIX timestamp (in seconds) for when the UCAN should expire.
    /// Setting this value overrides a configured lifetime value.
    pub fn with_expiration(mut self, timestamp: u64) -> Self {
        self.expiration = Some(timestamp);
        self
    }

    /// Set the POSIX timestamp (in seconds) of when the UCAN becomes active.
    pub fn not_before(mut self, timestamp: u64) -> Self {
        self.not_before = Some(timestamp);
        self
    }

    /// Add a fact or proof of knowledge to this UCAN.
    pub fn with_fact<T: Serialize + DeserializeOwned>(mut self, fact: T) -> Self {
        match serde_json::to_value(fact) {
            Ok(value) => self.facts.push(value),
            Err(error) => warn!("Could not add fact to UCAN: {}", error),
        }
        self
    }

    /// Will ensure that the built UCAN includes a number used once.
    pub fn with_nonce(mut self) -> Self {
        self.add_nonce = true;
        self
    }

    /// Includes a UCAN in the list of proofs for the UCAN to be built.
    /// Note that the proof's audience must match this UCAN's issuer
    /// or else the proof chain will be invalidated!
    pub fn witnessed_by(mut self, authority: &Ucan) -> Self {
        match authority.encode() {
            Ok(proof) => self.proofs.push(proof),
            Err(error) => warn!("Failed to add authority to proofs: {}", error),
        }

        self
    }

    /// Claim a capability by inheritance (from an authorizing proof) or
    /// implicitly by ownership of the resource by this UCAN's issuer
    pub fn claiming_capability<S, A>(mut self, capability: &Capability<S, A>) -> Self
    where
        S: Scope,
        A: Action,
    {
        let raw_capability: RawCapability = capability.clone().into();

        match serde_json::to_value(raw_capability) {
            Ok(value) => self.capabilities.push(value),
            Err(error) => warn!("UCAN could not claim capability: {}", error),
        }
        self
    }

    /// Delegate all capabilities from a given proof to the audience of the UCAN
    /// you're building
    pub fn delegating_from(mut self, authority: &Ucan) -> Self {
        match authority.encode() {
            Ok(proof) => {
                self.proofs.push(proof);
                let proof_index = self.proofs.len() - 1;
                let proof_delegation = ProofDelegationSemantics {};
                let capability =
                    proof_delegation.parse(format!("prf:{}", proof_index), "ucan/DELEGATE".into());

                match capability {
                    Some(capability) => {
                        let raw_capability: RawCapability = capability.into();
                        match serde_json::to_value(raw_capability) {
                            Ok(value) => self.capabilities.push(value),
                            Err(error) => {
                                warn!("Unable to convert capability to JSON value: {:?}", error);
                            }
                        }
                    }
                    None => warn!("Could not produce delegation capability"),
                }
            }
            Err(error) => warn!("Could not encode authoritative UCAN: {:?}", error),
        };

        self
    }

    fn implied_expiration(&self) -> Option<u64> {
        if self.expiration.is_some() {
            self.expiration
        } else {
            self.lifetime.map(|lifetime| now() + lifetime)
        }
    }

    pub fn build(self) -> Result<Signable<K>> {
        match &self.issuer {
            Some(issuer) => match &self.audience {
                Some(audience) => match self.implied_expiration() {
                    Some(expiration) => Ok(Signable {
                        issuer: issuer.clone(),
                        audience: audience.clone(),
                        not_before: self.not_before,
                        expiration,
                        facts: self.facts.clone(),
                        capabilities: self.capabilities.clone(),
                        proofs: self.proofs.clone(),
                        add_nonce: self.add_nonce,
                    }),
                    None => Err(anyhow!("Ambiguous lifetime")),
                },
                None => Err(anyhow!("Missing audience")),
            },
            None => Err(anyhow!("Missing issuer")),
        }
    }
}
