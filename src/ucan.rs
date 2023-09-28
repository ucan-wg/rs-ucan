//! JWT embedding of a UCAN

use std::str::FromStr;

use crate::{
    capability::{Capabilities, Capability, CapabilityParser, DefaultCapabilityParser},
    did_verifier::DidVerifierMap,
    error::Error,
    semantics::{ability::Ability, fact::DefaultFact, resource::Resource},
    time,
};
use cid::{
    multihash::{self, MultihashDigest},
    Cid,
};
use semver::Version;
use serde::{Deserialize, Deserializer, Serialize};

/// The current UCAN version
pub const UCAN_VERSION: &str = "0.10.0";

/// The UCAN header
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UcanHeader {
    pub(crate) alg: String,
    pub(crate) typ: String,
}

/// The UCAN payload
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct UcanPayload<F = DefaultFact, C = DefaultCapabilityParser> {
    pub(crate) ucv: String,
    pub(crate) iss: String,
    pub(crate) aud: String,
    #[serde(deserialize_with = "deserialize_required_nullable")]
    pub(crate) exp: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) nbf: Option<u64>,
    // TODO: nonce required in 1.0
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) nnc: Option<String>,
    #[serde(bound = "C: CapabilityParser")]
    pub(crate) cap: Capabilities<C>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) fct: Option<F>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub(crate) prf: Option<Vec<String>>,
}

/// A UCAN
#[derive(Clone, Debug)]
pub struct Ucan<F = DefaultFact, C = DefaultCapabilityParser> {
    pub(crate) header: jose_b64::serde::Json<UcanHeader>,
    pub(crate) payload: jose_b64::serde::Json<UcanPayload<F, C>>,
    pub(crate) signature: jose_b64::serde::Bytes,
}

impl<F, C> Ucan<F, C>
where
    F: Serialize,
    C: CapabilityParser,
{
    /// Validate the UCAN's signature and timestamps
    pub fn validate(
        &self,
        now_time: Option<u64>,
        did_verifier_map: &DidVerifierMap,
    ) -> Result<(), Error> {
        if self.typ() != "JWT" {
            return Err(Error::VerifyingError {
                msg: format!("expected header typ field to be 'JWT', got {}", self.typ()),
            });
        }

        if Version::parse(self.version()).is_err() {
            return Err(Error::VerifyingError {
                msg: format!(
                    "expected header ucv field to be a semver, got {}",
                    self.version()
                ),
            });
        }

        if self.is_expired(now_time) {
            return Err(Error::VerifyingError {
                msg: "token is expired".to_string(),
            });
        }

        if self.is_too_early() {
            return Err(Error::VerifyingError {
                msg: "current time is before token validity period begins".to_string(),
            });
        }

        // TODO: parse and validate iss and aud DIDs during deserialization
        self.payload
            .aud
            .strip_prefix("did:")
            .and_then(|did| did.split_once(':'))
            .ok_or(Error::VerifyingError {
                msg: format!(
                    "expected did:<method>:<identifier>, got {}",
                    self.payload.aud
                ),
            })?;

        let (method, identifier) = self
            .payload
            .iss
            .strip_prefix("did:")
            .and_then(|did| did.split_once(':'))
            .ok_or(Error::VerifyingError {
                msg: format!(
                    "expected did:<method>:<identifier>, got {}",
                    self.payload.iss
                ),
            })?;

        let header = serde_json::to_value(&self.header)
            .map_err(|e| Error::InternalUcanError { msg: e.to_string() })?;

        let payload = serde_json::to_value(&self.payload)
            .map_err(|e| Error::InternalUcanError { msg: e.to_string() })?;

        let signed_data = format!(
            "{}.{}",
            header.as_str().ok_or(Error::InternalUcanError {
                msg: "Expected base64 encoding of header".to_string(),
            })?,
            payload.as_str().ok_or(Error::InternalUcanError {
                msg: "Expected base64 encoding of payload".to_string(),
            })?,
        );

        did_verifier_map.verify(method, identifier, signed_data.as_bytes(), &self.signature)
    }

    /// Encode the UCAN as a JWT token
    pub fn encode(&self) -> Result<String, Error> {
        let header = serde_json::to_value(&self.header)
            .map_err(|e| Error::InternalUcanError { msg: e.to_string() })?;

        let payload = serde_json::to_value(&self.payload)
            .map_err(|e| Error::InternalUcanError { msg: e.to_string() })?;

        let signature = serde_json::to_value(&self.signature)
            .map_err(|e| Error::InternalUcanError { msg: e.to_string() })?;

        Ok(format!(
            "{}.{}.{}",
            header.as_str().ok_or(Error::InternalUcanError {
                msg: "Expected base64 encoding of header".to_string(),
            })?,
            payload.as_str().ok_or(Error::InternalUcanError {
                msg: "Expected base64 encoding of payload".to_string(),
            })?,
            signature.as_str().ok_or(Error::InternalUcanError {
                msg: "Expected base64 encoding of signature".to_string(),
            })?
        ))
    }

    /// Returns true if the UCAN has past its expiration date
    pub fn is_expired(&self, now_time: Option<u64>) -> bool {
        if let Some(exp) = self.payload.exp {
            exp < now_time.unwrap_or_else(time::now)
        } else {
            false
        }
    }

    /// Returns the UCAN's signature
    pub fn signature(&self) -> &jose_b64::serde::Bytes {
        &self.signature
    }

    /// Returns true if the not-before ("nbf") time is still in the future
    pub fn is_too_early(&self) -> bool {
        match self.payload.nbf {
            Some(nbf) => nbf > time::now(),
            None => false,
        }
    }

    /// Returns true if this UCAN's lifetime begins no later than the other
    /// Note that if a UCAN specifies an NBF but the other does not, the
    /// other has an unbounded start time and this function will return
    /// false.
    pub fn lifetime_begins_before(&self, other: &Ucan<C, F>) -> bool {
        match (self.payload.nbf, other.payload.nbf) {
            (Some(nbf), Some(other_nbf)) => nbf <= other_nbf,
            (Some(_), None) => false,
            _ => true,
        }
    }

    /// Returns true if this UCAN expires no earlier than the other
    pub fn lifetime_ends_after(&self, other: &Ucan<C, F>) -> bool {
        match (self.payload.exp, other.payload.exp) {
            (Some(exp), Some(other_exp)) => exp >= other_exp,
            (Some(_), None) => false,
            (None, _) => true,
        }
    }

    /// Returns true if this UCAN's lifetime fully encompasses the other
    pub fn lifetime_encompasses(&self, other: &Ucan<C, F>) -> bool {
        self.lifetime_begins_before(other) && self.lifetime_ends_after(other)
    }

    /// Return the `typ` field of the UCAN header
    pub fn typ(&self) -> &str {
        &self.header.typ
    }

    /// Return the `alg` field of the UCAN header
    pub fn algorithm(&self) -> &str {
        &self.header.alg
    }

    /// Return the `iss` field of the UCAN payload
    pub fn issuer(&self) -> &str {
        &self.payload.iss
    }

    /// Return the `aud` field of the UCAN payload
    pub fn audience(&self) -> &str {
        &self.payload.aud
    }

    /// Return the `prf` field of the UCAN payload
    pub fn proofs(&self) -> Option<&Vec<String>> {
        self.payload.prf.as_ref()
    }

    /// Return the `exp` field of the UCAN payload
    pub fn expires_at(&self) -> Option<u64> {
        self.payload.exp
    }

    /// Return the `nbf` field of the UCAN payload
    pub fn not_before(&self) -> Option<u64> {
        self.payload.nbf
    }

    /// Return the `nnc` field of the UCAN payload
    pub fn nonce(&self) -> Option<&String> {
        self.payload.nnc.as_ref()
    }

    /// Return an iterator over the `cap` field of the UCAN payload
    pub fn capabilities(&self) -> impl Iterator<Item = &'_ Capability> {
        self.payload.cap.iter()
    }

    /// Return the `fct` field of the UCAN payload
    pub fn facts(&self) -> Option<&F> {
        self.payload.fct.as_ref()
    }

    /// Return the `ucv` field of the UCAN payload
    pub fn version(&self) -> &str {
        &self.payload.ucv
    }

    /// Return the CID v1 of the UCAN encoded as a JWT token
    pub fn to_cid(&self, hasher: multihash::Code) -> Result<Cid, Error> {
        static RAW_CODEC: u64 = 0x55;

        let token = self.encode()?;
        let digest = hasher.digest(token.as_bytes());
        let cid = Cid::new_v1(RAW_CODEC, digest);

        Ok(cid)
    }

    /// Returns true if the UCAN authorizes the given resource and ability
    // TODO: This is an old placeholder implementation that needs to take
    // into account the issuer of the capabilities
    pub fn is_authorized<R, A>(&self, resource: &R, ability: &A) -> bool
    where
        R: Resource,
        A: Ability,
    {
        for capability in self.capabilities() {
            if !resource.is_valid_attenuation(capability.resource()) {
                continue;
            }

            if !ability.is_valid_attenuation(capability.ability()) {
                continue;
            }

            if capability.caveat().is_valid() {
                return true;
            }
        }

        false
    }
}

impl<'a> TryFrom<&'a str> for Ucan {
    type Error = Error;

    fn try_from(ucan_token: &str) -> Result<Self, Self::Error> {
        Ucan::from_str(ucan_token)
    }
}

impl TryFrom<String> for Ucan {
    type Error = Error;

    fn try_from(ucan_token: String) -> Result<Self, Self::Error> {
        Ucan::from_str(ucan_token.as_str())
    }
}

impl FromStr for Ucan {
    type Err = Error;

    fn from_str(ucan_token: &str) -> Result<Self, Self::Err> {
        let &[header, payload, signature] =
            ucan_token.splitn(3, '.').collect::<Vec<_>>().as_slice()
        else {
            return Err(Error::TokenParseError {
                msg: "malformed token, expected 3 parts separated by dots".to_string(),
            });
        };

        let header =
            jose_b64::serde::Json::from_str(header).map_err(|_| Error::TokenParseError {
                msg: "malformed header".to_string(),
            })?;

        let payload =
            jose_b64::serde::Json::from_str(payload).map_err(|_| Error::TokenParseError {
                msg: "malformed payload".to_string(),
            })?;

        let signature =
            jose_b64::serde::Bytes::from_str(signature).map_err(|_| Error::TokenParseError {
                msg: "malformed signature".to_string(),
            })?;

        Ok(Ucan {
            header,
            payload,
            signature,
        })
    }
}

fn deserialize_required_nullable<'de, T, D>(deserializer: D) -> Result<T, D::Error>
where
    T: Deserialize<'de>,
    D: Deserializer<'de>,
{
    Deserialize::deserialize(deserializer)
        .map_err(|_| serde::de::Error::custom("required field is missing or has invalid type"))
}
