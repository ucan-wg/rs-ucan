// use crate::{Error, Unit};
use enum_as_inner::EnumAsInner;
use libipld_core::{ipld::Ipld, multibase::Base::Base32HexLower};
use serde::{Deserialize, Serialize};
use std::fmt;
use uuid::Uuid;

// FIXME
pub struct Unit;
// FIXME
pub struct Error<T>(T);

/// Enumeration over allowed `nonce` types.
#[derive(Clone, Debug, PartialEq, EnumAsInner, Serialize, Deserialize)]
pub enum Nonce {
    /// 96-bit, 12-byte nonce, e.g. [`xid`].
    Nonce96([u8; 12]),
    /// 128-bit, 16-byte nonce.
    Nonce128([u8; 16]),
    /// No Nonce attributed.
    Custom(Vec<u8>),
}

impl Nonce {
    /// Default generator, outputting an [`xid`] nonce,
    /// which is a 96-bit (12-byte) nonce.
    pub fn generate() -> Self {
        Nonce::Nonce96(*xid::new().as_bytes())
    }

    /// Generate a default 128-bit(16-byte) nonce via [`Uuid::new_v4()`].
    pub fn generate_128() -> Self {
        Nonce::Nonce128(*Uuid::new_v4().as_bytes())
    }
}

impl fmt::Display for Nonce {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Nonce::Nonce96(nonce) => {
                write!(f, "{}", Base32HexLower.encode(nonce.as_slice()))
            }
            Nonce::Nonce128(nonce) => {
                write!(f, "{}", Base32HexLower.encode(nonce.as_slice()))
            }
            Nonce::Custom(nonce) => {
                write!(f, "{}", Base32HexLower.encode(nonce.as_slice()))
            }
        }
    }
}

impl From<Nonce> for Ipld {
    fn from(nonce: Nonce) -> Self {
        match nonce {
            Nonce::Nonce96(nonce) => Ipld::Bytes(nonce.to_vec()),
            Nonce::Nonce128(nonce) => Ipld::Bytes(nonce.to_vec()),
            Nonce::Custom(nonce) => Ipld::Bytes(nonce),
        }
    }
}

impl TryFrom<Ipld> for Nonce {
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        if let Ipld::Bytes(v) = ipld {
            match v.len() {
                12 => Ok(Nonce::Nonce96(
                    v.try_into()
                        .expect("12 bytes because we checked in the match"),
                )),
                16 => Ok(Nonce::Nonce128(
                    v.try_into()
                        .expect("16 bytes because we checked in the match"),
                )),
                _ => Ok(Nonce::Custom(v)),
            }
        } else {
            Err(())
        }
    }
}

impl TryFrom<&Ipld> for Nonce {
    type Error = (); // FIXME

    fn try_from(ipld: &Ipld) -> Result<Self, Self::Error> {
        TryFrom::try_from(ipld.to_owned())
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn ipld_roundtrip_12() {
        let gen = Nonce::generate();
        let ipld = Ipld::from(gen.clone());

        let inner = if let Nonce::Nonce96(nonce) = gen {
            Ipld::Bytes(nonce.to_vec())
        } else {
            panic!("No conversion!")
        };

        assert_eq!(ipld, inner);
        assert_eq!(gen, ipld.try_into().unwrap());
    }

    #[test]
    fn ipld_roundtrip_16() {
        let gen = Nonce::generate_128();
        let ipld = Ipld::from(gen.clone());

        let inner = if let Nonce::Nonce128(nonce) = gen {
            Ipld::Bytes(nonce.to_vec())
        } else {
            panic!("No conversion!")
        };

        assert_eq!(ipld, inner);
        assert_eq!(gen, ipld.try_into().unwrap());
    }

    #[test]
    fn ser_de() {
        let gen = Nonce::generate_128();
        let ser = serde_json::to_string(&gen).unwrap();
        let de = serde_json::from_str(&ser).unwrap();

        assert_eq!(gen, de);
    }
}
