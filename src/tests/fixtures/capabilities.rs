use serde::{Deserialize, Serialize};

use crate::capability::Capability;

#[derive(PartialEq, PartialOrd)]
pub enum WNFSCapLevel {
    None,
    Create,
    Revise,
    SoftDelete,
    Overwrite,
    SuperUser,
}

impl ToString for WNFSCapLevel {
    fn to_string(&self) -> String {
        match self {
            &WNFSCapLevel::SuperUser => "SUPER_USER",
            &WNFSCapLevel::Overwrite => "OVERWRITE",
            &WNFSCapLevel::SoftDelete => "SOFT_DELETE",
            &WNFSCapLevel::Revise => "REVISE",
            &WNFSCapLevel::Create => "CREATE",
            _ => "NONE",
        }
        .into()
    }
}

impl From<&String> for WNFSCapLevel {
    fn from(value: &String) -> Self {
        match value.as_str() {
            "SUPER_USER" => WNFSCapLevel::SuperUser,
            "OVERWRITE" => WNFSCapLevel::Overwrite,
            "SOFT_DELETE" => WNFSCapLevel::SoftDelete,
            "REVISE" => WNFSCapLevel::Revise,
            "CREATE" => WNFSCapLevel::Create,
            _ => WNFSCapLevel::None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct WNFSCapability {
    pub wnfs: String,
    pub cap: String,
}

impl WNFSCapability {
    pub fn user(&self) -> String {
        if let Some(host) = self.wnfs.split('/').nth(0) {
            if let Some(user) = host.split('.').nth(0) {
                return String::from(user);
            }
        }

        String::from("")
    }

    pub fn public_path(&self) -> Vec<String> {
        self.wnfs
            .split('/')
            .enumerate()
            .filter_map(|(index, part)| match index {
                0..=1 => None,
                _ => Some(String::from(part)),
            })
            .collect()
    }
}

impl Capability for WNFSCapability {
    fn delegate_to(&self, other: &Self) -> Option<Self> {
        if self.user() != other.user() {
            return None;
        }

        if WNFSCapLevel::from(&other.cap) > WNFSCapLevel::from(&self.cap) {
            todo!("Escalation")
        }

        let other_path = other.public_path();
        let self_path = self.public_path();

        if other_path.len() < self_path.len() {
            todo!("Escalation")
        }

        for (self_part, other_part) in self_path.iter().zip(other_path.iter()) {
            if self_part != other_part {
                todo!("Escalation")
            }
        }

        Some(WNFSCapability {
            wnfs: other.wnfs.clone(),
            cap: other.cap.clone(),
        })
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct EmailCapability {
    pub email: String,
    pub cap: String,
}

impl Capability for EmailCapability {
    fn delegate_to(&self, other: &Self) -> Option<Self> {
        match self.email == other.email {
            true => Some(EmailCapability {
                email: other.email.clone(),
                cap: other.cap.clone(),
            }),
            false => None,
        }
    }
}
