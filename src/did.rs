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
    type Error = <DID as TryFrom<String>>::Error;

    fn try_from(string: String) -> Result<Self, Self::Error> {
        DID::parse(&string).map(Did)
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
    type Error = <Did as TryFrom<String>>::Error; // FIXME also include the "can't parse form ipld" case; seems like someythjing taht can be abstrcated out, too

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::String(string) => Did::try_from(string),
            _ => todo!(), // Err(()),
        }
    }
}
