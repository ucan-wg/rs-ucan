//! [Varsig] implementation.
//!
//! This includes both signature metadata and helpers for signing, verifying,
//! and encoding payloads per a given [Varsig] configuration.
//!
//! [Varsig]: https://github.com/ChainAgnostic/varsig
//!
//! # Example
//!
//! ```rust
//! use varsig::{Varsig, signature::eddsa::Ed25519};
//! use serde_ipld_dagcbor::codec::DagCborCodec;
//! use serde::{Serialize, Deserialize};
//!
//! // Your data type
//! #[derive(Debug, Serialize, Deserialize)]
//! struct Character {
//!     name: String,
//!     hp: u16,
//!     mp: u16,
//! }
//!
//! let payload = Character {
//!     name: "Terra Branford".to_string(),
//!     hp: 100,
//!     mp: 20,
//! };
//!
//! // ✨ Varsig configuration for Ed25519 and DAG-CBOR ✨
//! let varsig: Varsig<Ed25519, DagCborCodec, Character> = Varsig::default();
//!
//! // Signing the payload with enforced Ed25519 and DAG-CBOR
//! let sk = ed25519_dalek::SigningKey::generate(&mut rand::thread_rng());
//! let (sig, _) = varsig.try_sign(&sk, &payload).unwrap();
//! varsig.try_verify(&sk.verifying_key(), &payload, &sig).unwrap();
//! ```

#![allow(clippy::multiple_crate_versions)] // syn
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod codec;
pub mod curve;
pub mod encoding;
pub mod hash;
pub mod header;
pub mod signature;
pub mod signer;
pub mod verify;

pub use header::Varsig;
