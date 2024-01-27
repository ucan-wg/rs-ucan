use did_url::DID;
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(into = "String", try_from = "String")]
pub struct Did(DID);

impl From<Did> for String {
    fn from(did: Did) -> Self {
        did.0.to_string()
    }
}

impl TryFrom<String> for Did {
    type Error = String; // FIXME

    fn try_from(string: String) -> Result<Self, Self::Error> {
        DID::parse(&string)
            .map_err(|err| format!("Failed to parse DID: {}", err))
            .map(Self)
    }
}

impl fmt::Display for Did {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

impl From<Did> for Ipld {
    fn from(did: Did) -> Self {
        did.into()
    }
}

impl TryFrom<Ipld> for Did {
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        Self::try_from(ipld)
    }
}
