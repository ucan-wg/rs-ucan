use anyhow::{anyhow, Result};
use url::Url;

use super::{Action, CapabilitySemantics, Scope};

#[derive(Ord, Eq, PartialEq, PartialOrd, Clone)]
pub enum ProofAction {
    Delegate,
}

impl Action for ProofAction {}

impl TryFrom<String> for ProofAction {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self> {
        match value.as_str() {
            "ucan/DELEGATE" => Ok(ProofAction::Delegate),
            unsupported => Err(anyhow!(
                "Unsupported action for proof resource ({})",
                unsupported
            )),
        }
    }
}

impl ToString for ProofAction {
    fn to_string(&self) -> String {
        match self {
            ProofAction::Delegate => "ucan/DELEGATE".into(),
        }
    }
}

#[derive(PartialEq, Clone)]
pub enum ProofSelection {
    Index(usize),
    All,
}

impl Scope for ProofSelection {
    fn contains(&self, other: &Self) -> bool {
        return self == other || *self == ProofSelection::All;
    }
}

impl TryFrom<Url> for ProofSelection {
    type Error = anyhow::Error;

    fn try_from(value: Url) -> Result<Self, Self::Error> {
        match value.scheme() {
            "prf" => match value.path() {
                scope => String::from(scope).try_into(),
            },
            _ => Err(anyhow!("Unrecognized URI scheme")),
        }
    }
}

impl TryFrom<String> for ProofSelection {
    type Error = anyhow::Error;

    fn try_from(value: String) -> Result<Self> {
        match value.as_str() {
            "*" => Ok(ProofSelection::All),
            selection => Ok(ProofSelection::Index(usize::from_str_radix(selection, 10)?)),
        }
    }
}

impl ToString for ProofSelection {
    fn to_string(&self) -> String {
        match self {
            ProofSelection::Index(usize) => format!("prf:{}", usize),
            ProofSelection::All => format!("prf:*"),
        }
    }
}

pub struct ProofDelegationSemantics {}

impl CapabilitySemantics<ProofSelection, ProofAction> for ProofDelegationSemantics {}
