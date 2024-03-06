//! Wasm module representations

use crate::ipld;
use base64::{display::Base64Display, engine::general_purpose::STANDARD, Engine as _};
use libipld_core::{cid::Cid, error::SerdeError, ipld::Ipld, link::Link, serde as ipld_serde};
use serde::{Deserialize, Serialize};

/// Ways to represent a Wasm module in a `wasm/run` payload.
#[derive(Debug, Clone, PartialEq)]
pub enum Module {
    /// The raw bytes of the Wasm module
    ///
    /// Encodes as a `data:` URL
    Inline(Vec<u8>),

    /// A [`Cid`] link to the Wasm module
    Remote(Link<Vec<u8>>),
}

impl From<Module> for Ipld {
    fn from(module: Module) -> Self {
        match module {
            Module::Inline(bytes) => Ipld::Bytes(bytes),
            Module::Remote(cid) => Ipld::Link(*cid),
        }
    }
}

impl TryFrom<ipld::Newtype> for Module {
    type Error = SerdeError;

    fn try_from(nt: ipld::Newtype) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(nt.0)
    }
}

impl TryFrom<Ipld> for Module {
    type Error = SerdeError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        ipld_serde::from_ipld(ipld)
    }
}

impl Serialize for Module {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::ser::Serializer,
    {
        match self {
            Module::Remote(link) => link.cid().serialize(serializer),
            Module::Inline(bytes) => format!(
                "data:application/wasm;base64,{}",
                Base64Display::new(bytes.as_ref(), &STANDARD)
            )
            .serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Module {
    fn deserialize<D>(deserializer: D) -> Result<Module, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        if s.starts_with("data:") {
            let data = s
                .split(',')
                .nth(1)
                .ok_or_else(|| serde::de::Error::custom("missing base64 data"))?;

            let bytes = STANDARD
                .decode(data)
                .map_err(|err| serde::de::Error::custom(err))?;

            Ok(Module::Inline(bytes))
        } else {
            let cid = Cid::try_from(s).map_err(serde::de::Error::custom)?;
            Ok(Module::Remote(Link::new(cid)))
        }
    }
}

impl From<Module> for ipld::Promised {
    fn from(module: Module) -> Self {
        module.into()
    }
}
