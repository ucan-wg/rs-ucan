use std::io::Cursor;

use anyhow::Result;
use libipld_core::ipld::Ipld;
use libipld_core::serde::from_ipld;
use libipld_core::{
    codec::{Decode, Encode},
    serde::to_ipld,
};
use libipld_json::DagJsonCodec;
use serde::{de::DeserializeOwned, Serialize, Serializer};

/// Utility function to enforce lower-case string values when serializing
pub fn ser_to_lower_case<S>(string: &String, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&string.to_lowercase())
}

/// Helper trait to ser/de any serde-implementing value to/from DAG-JSON
pub trait DagJson: Serialize + DeserializeOwned {
    fn to_dag_json(&self) -> Result<Vec<u8>> {
        let ipld = to_ipld(self)?;
        let mut json_bytes = Vec::new();

        ipld.encode(DagJsonCodec, &mut json_bytes)?;

        Ok(json_bytes)
    }

    fn from_dag_json(json_bytes: &[u8]) -> Result<Self> {
        let ipld = Ipld::decode(DagJsonCodec, &mut Cursor::new(json_bytes))?;
        Ok(from_ipld(ipld)?)
    }
}

impl<T> DagJson for T where T: Serialize + DeserializeOwned {}

/// Helper trait to encode structs as base64 as part of creating a JWT
pub trait Base64Encode: DagJson {
    fn jwt_base64_encode(&self) -> Result<String> {
        Ok(base64::encode_config(
            self.to_dag_json()?,
            base64::URL_SAFE_NO_PAD,
        ))
    }
}

impl<T> Base64Encode for T where T: DagJson {}
