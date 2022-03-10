use anyhow::{anyhow, Result};

use crate::capability::{Action, CapabilitySemantics, Scope};
use url::Url;

#[derive(Ord, Eq, PartialOrd, PartialEq, Clone)]
pub enum WNFSCapLevel {
    Create,
    Revise,
    SoftDelete,
    Overwrite,
    SuperUser,
}

impl Action for WNFSCapLevel {}

impl TryFrom<String> for WNFSCapLevel {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self> {
        Ok(match value.as_str() {
            "wnfs/CREATE" => WNFSCapLevel::Create,
            "wnfs/REVISE" => WNFSCapLevel::Revise,
            "wnfs/SOFT_DELETE" => WNFSCapLevel::SoftDelete,
            "wnfs/OVERWRITE" => WNFSCapLevel::Overwrite,
            "wnfs/SUPER_USER" => WNFSCapLevel::SuperUser,
            _ => return Err(anyhow!("No such WNFS capability level: {}", value)),
        })
    }
}

impl ToString for WNFSCapLevel {
    fn to_string(&self) -> String {
        match self {
            WNFSCapLevel::Create => "wnfs/CREATE",
            WNFSCapLevel::Revise => "wnfs/REVISE",
            WNFSCapLevel::SoftDelete => "wnfs/SOFT_DELETE",
            WNFSCapLevel::Overwrite => "wnfs/OVERWRITE",
            WNFSCapLevel::SuperUser => "wnfs/SUPER_USER",
        }
        .into()
    }
}

#[derive(Clone, PartialEq)]
pub struct WNFSScope {
    origin: String,
    path: String,
}

impl Scope for WNFSScope {
    fn contains(&self, other: &Self) -> bool {
        if self.origin != other.origin {
            return false;
        }

        let self_path_parts = self.path.split('/');
        let mut other_path_parts = other.path.split('/');

        for part in self_path_parts {
            match other_path_parts.nth(0) {
                Some(other_part) => {
                    if part != other_part {
                        return false;
                    }
                }
                None => return false,
            }
        }

        true
    }
}

impl TryFrom<Url> for WNFSScope {
    type Error = anyhow::Error;

    fn try_from(value: Url) -> Result<Self, Self::Error> {
        match (value.scheme(), value.host_str(), value.path()) {
            ("wnfs", Some(host), path) => Ok(WNFSScope {
                origin: String::from(host),
                path: String::from(path),
            }),
            _ => Err(anyhow!("Cannot interpret URI as WNFS scope: {}", value)),
        }
    }
}

impl ToString for WNFSScope {
    fn to_string(&self) -> String {
        format!("wnfs://{}{}", self.origin, self.path)
    }
}

pub struct WNFSSemantics {}

impl CapabilitySemantics<WNFSScope, WNFSCapLevel> for WNFSSemantics {}
