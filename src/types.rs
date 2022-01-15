use crate::chain::ProofChain;
// use anyhow::{anyhow, Result};
use serde_json::Value;
use std::collections::BTreeMap as Map;

// pub struct BMap<String, Value>);

pub type Fact = Map<String, Value>;
pub type Capability = Map<String, String>;

// impl From<Value> for Fact {
//     fn from(value: Value) -> Self {
//         match value {
//             Value::Object(fact) => Fact(fact),
//             _ => Fact(Map::new()),
//         }
//     }
// }

// Placeholder types:

pub type CapabilitySemantics = ();

pub type UcanStore = ();

pub enum CapabilityAuthority {
    Proof(ProofChain),
    Store(UcanStore),
}
