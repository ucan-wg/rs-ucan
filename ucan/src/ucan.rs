use std::convert::TryFrom;

use anyhow::{anyhow, Context, Result};
use cid::multihash::{Code, MultihashDigest};
use cid::Cid;
use libipld_core::codec::Codec;
use libipld_core::raw::RawCodec;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::capability::CapabilityIpld;
use crate::crypto::did::DidParser;
use crate::serde::{Base64Encode, DagJson};
use crate::time::now;

pub const UCAN_VERSION: &'static str = "0.9.0-canary";

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UcanHeader {
    pub alg: String,
    pub typ: String,
    pub ucv: String,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
pub struct UcanPayload {
    pub iss: String,
    pub aud: String,
    pub exp: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nbf: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nnc: Option<String>,
    pub att: Vec<CapabilityIpld>,
    pub fct: Vec<Value>,
    pub prf: Vec<String>,
}

#[derive(Clone, Serialize, Deserialize, Debug)]
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

    /// Deserialize an encoded UCAN token string into a UCAN
    pub fn try_from_token_string(ucan_token_string: &str) -> Result<Ucan> {
        let signed_data = ucan_token_string
            .split('.')
            .take(2)
            .map(String::from)
            .reduce(|l, r| format!("{}.{}", l, r))
            .ok_or_else(|| anyhow!("Could not parse signed data from token string"))?;

        let mut parts = ucan_token_string.split('.').map(|str| {
            base64::decode_config(str, base64::URL_SAFE_NO_PAD).map_err(|error| anyhow!(error))
        });

        let header: UcanHeader = match parts.next() {
            Some(part) => match part {
                Ok(decoded) => match UcanHeader::from_dag_json(&decoded) {
                    Ok(header) => header,
                    Err(error) => return Err(error).context("Could not parse UCAN header JSON"),
                },
                Err(error) => return Err(error).context("Could not decode UCAN header base64"),
            },
            None => return Err(anyhow!("Missing UCAN header in token part")),
        };

        let payload: UcanPayload = match parts.next() {
            Some(part) => match part {
                Ok(decoded) => match UcanPayload::from_dag_json(&decoded) {
                    Ok(payload) => payload,
                    Err(error) => return Err(error).context("Could not parse UCAN payload JSON"),
                },
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

        Ok(Ucan::new(
            header,
            payload,
            signed_data.as_bytes().into(),
            signature,
        ))
    }

    /// Validate the UCAN's signature and timestamps
    pub async fn validate<'a>(&self, did_parser: &mut DidParser) -> Result<()> {
        if self.is_expired() {
            return Err(anyhow!("Expired"));
        }

        if self.is_too_early() {
            return Err(anyhow!("Not active yet (too early)"));
        }

        self.check_signature(did_parser).await
    }

    /// Validate that the signed data was signed by the stated issuer
    pub async fn check_signature<'a>(&self, did_parser: &mut DidParser) -> Result<()> {
        let key = did_parser.parse(&self.payload.iss)?;
        key.verify(&self.signed_data, &self.signature).await
    }

    /// Produce a base64-encoded serialization of the UCAN suitable for
    /// transferring in a header field
    pub fn encode(&self) -> Result<String> {
        let header = self.header.jwt_base64_encode()?;
        let payload = self.payload.jwt_base64_encode()?;
        let signature = base64::encode_config(self.signature.as_slice(), base64::URL_SAFE_NO_PAD);

        Ok(format!("{}.{}.{}", header, payload, signature))
    }

    /// Returns true if the UCAN has past its expiration date
    pub fn is_expired(&self) -> bool {
        self.payload.exp < now()
    }

    /// Raw bytes of signed data for this UCAN
    pub fn signed_data(&self) -> &[u8] {
        &self.signed_data
    }

    pub fn signature(&self) -> &[u8] {
        &self.signature
    }

    /// Returns true if the not-before ("nbf") time is still in the future
    pub fn is_too_early(&self) -> bool {
        match self.payload.nbf {
            Some(nbf) => nbf > now(),
            None => false,
        }
    }

    /// Returns true if this UCAN's lifetime begins no later than the other
    /// Note that if a UCAN specifies an NBF but the other does not, the
    /// other has an unbounded start time and this function will return
    /// false.
    pub fn lifetime_begins_before(&self, other: &Ucan) -> bool {
        match (self.payload.nbf, other.payload.nbf) {
            (Some(nbf), Some(other_nbf)) => nbf <= other_nbf,
            (Some(_), None) => false,
            _ => true,
        }
    }

    /// Returns true if this UCAN expires no earlier than the other
    pub fn lifetime_ends_after(&self, other: &Ucan) -> bool {
        self.payload.exp >= other.payload.exp
    }

    /// Returns true if this UCAN's lifetime fully encompasses the other
    pub fn lifetime_encompasses(&self, other: &Ucan) -> bool {
        self.lifetime_begins_before(other) && self.lifetime_ends_after(other)
    }

    pub fn algorithm(&self) -> &str {
        &self.header.alg
    }

    pub fn issuer(&self) -> &str {
        &self.payload.iss
    }

    pub fn audience(&self) -> &str {
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

    pub fn attenuation(&self) -> &Vec<CapabilityIpld> {
        &self.payload.att
    }

    pub fn facts(&self) -> &Vec<Value> {
        &self.payload.fct
    }

    pub fn version(&self) -> &str {
        &self.header.ucv
    }
}

impl TryFrom<&Ucan> for Cid {
    type Error = anyhow::Error;

    fn try_from(value: &Ucan) -> Result<Self, Self::Error> {
        let codec = RawCodec::default();
        let token = value.encode()?;
        let encoded = codec.encode(token.as_bytes())?;

        Ok(Cid::new_v1(codec.into(), Code::Blake2b256.digest(&encoded)))
    }
}

impl TryFrom<Ucan> for Cid {
    type Error = anyhow::Error;

    fn try_from(value: Ucan) -> Result<Self, Self::Error> {
        Cid::try_from(&value)
    }
}
