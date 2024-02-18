//! [Nonce]s & utilities.
//!
//! [Nonce]: https://en.wikipedia.org/wiki/Cryptographic_nonce

// FIXME use enum_as_inner more?
use enum_as_inner::EnumAsInner;
use getrandom::getrandom;
use libipld_core::{
    ipld::Ipld,
    multibase::Base::Base32HexLower,
    multihash::{Hasher, Sha2_256},
};
use serde::{Deserialize, Serialize};
use std::fmt;

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(feature = "test_utils")]
use proptest::prelude::*;

/// Known [`Nonce`] types
#[derive(Clone, Debug, PartialEq, EnumAsInner, Serialize, Deserialize)]
pub enum Nonce {
    /// 96-bit, 12-byte nonce
    Nonce12([u8; 12]),

    /// 128-bit, 16-byte nonce
    Nonce16([u8; 16]),

    /// Dynamic sized nonce
    Custom(Vec<u8>),
}

impl From<[u8; 12]> for Nonce {
    fn from(s: [u8; 12]) -> Self {
        Nonce::Nonce12(s)
    }
}

impl From<[u8; 16]> for Nonce {
    fn from(s: [u8; 16]) -> Self {
        Nonce::Nonce16(s)
    }
}

impl From<Nonce> for Vec<u8> {
    fn from(nonce: Nonce) -> Self {
        match nonce {
            Nonce::Nonce12(nonce) => nonce.to_vec(),
            Nonce::Nonce16(nonce) => nonce.to_vec(),
            Nonce::Custom(nonce) => nonce,
        }
    }
}

impl From<Vec<u8>> for Nonce {
    fn from(nonce: Vec<u8>) -> Self {
        match nonce.len() {
            12 => Nonce::Nonce12(
                nonce
                    .try_into()
                    .expect("12 bytes because we checked in the match"),
            ),
            16 => Nonce::Nonce16(
                nonce
                    .try_into()
                    .expect("16 bytes because we checked in the match"),
            ),
            _ => Nonce::Custom(nonce),
        }
    }
}

impl Nonce {
    /// Generate a 96-bit, 12-byte nonce.
    /// This is the minimum nonce size typically recommended.
    ///
    /// # Arguments
    ///
    /// * `salt` - A salt. This may be left empty, but is recommended to avoid collision.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use ucan::crypto::Nonce;
    /// # use ucan::did::Did;
    /// #
    /// let mut salt = "did:example:123".as_bytes().to_vec();
    /// let nonce = Nonce::generate_12(&mut salt);
    ///
    /// assert_eq!(Vec::from(nonce).len(), 12);
    /// ```
    pub fn generate_12(salt: &mut Vec<u8>) -> Nonce {
        salt.append(&mut [0].repeat(12));

        let buf = salt.as_mut_slice();
        getrandom(buf).expect("irrecoverable getrandom failure");

        let mut hasher = Sha2_256::default();
        hasher.update(buf);

        let bytes = hasher
            .finalize()
            .chunks(12)
            .next()
            .expect("SHA2_256 is 32 bytes")
            .try_into()
            .expect("we set the length to 12 earlier");

        Nonce::Nonce12(bytes)
    }

    /// Generate a 128-bit, 16-byte nonce
    ///
    /// # Arguments
    ///
    /// * `salt` - A salt. This may be left empty, but is recommended to avoid collision.
    ///
    /// # Example
    ///
    /// ```rust
    /// # use ucan::crypto::Nonce;
    /// # use ucan::did::Did;
    /// #
    /// let mut salt = "did:example:123".as_bytes().to_vec();
    /// let nonce = Nonce::generate_16(&mut salt);
    ///
    /// assert_eq!(Vec::from(nonce).len(), 16);
    /// ```
    pub fn generate_16(salt: &mut Vec<u8>) -> Nonce {
        salt.append(&mut [0].repeat(16));

        let buf = salt.as_mut_slice();
        getrandom(buf).expect("irrecoverable getrandom failure");

        let mut hasher = Sha2_256::default();
        hasher.update(buf);

        let bytes = hasher
            .finalize()
            .chunks(16)
            .next()
            .expect("SHA2_256 is 32 bytes")
            .try_into()
            .expect("we set the length to 16 earlier");

        Nonce::Nonce16(bytes)
    }
}

impl fmt::Display for Nonce {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Nonce::Nonce12(nonce) => {
                write!(f, "{}", Base32HexLower.encode(nonce.as_slice()))
            }
            Nonce::Nonce16(nonce) => {
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
            Nonce::Nonce12(nonce) => Ipld::Bytes(nonce.to_vec()),
            Nonce::Nonce16(nonce) => Ipld::Bytes(nonce.to_vec()),
            Nonce::Custom(nonce) => Ipld::Bytes(nonce),
        }
    }
}

impl TryFrom<Ipld> for Nonce {
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        if let Ipld::Bytes(v) = ipld {
            match v.len() {
                12 => Ok(Nonce::Nonce12(
                    v.try_into()
                        .expect("12 bytes because we checked in the match"),
                )),
                16 => Ok(Nonce::Nonce16(
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

#[cfg(feature = "test_utils")]
impl Arbitrary for Nonce {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        prop_oneof![
            any::<[u8; 12]>().prop_map(Nonce::Nonce12),
            any::<[u8; 16]>().prop_map(Nonce::Nonce16),
            any::<Vec<u8>>().prop_map(Nonce::Custom)
        ]
        .boxed()
    }
}

// FIXME move module?
#[cfg(target_arch = "wasm32")]
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[wasm_bindgen]
/// A JavaScript-compatible wrapper for [`Nonce`]
pub struct JsNonce(#[wasm_bindgen(skip)] pub Nonce);

#[cfg(target_arch = "wasm32")]
impl From<JsNonce> for Nonce {
    fn from(newtype: JsNonce) -> Self {
        newtype.0
    }
}

#[cfg(target_arch = "wasm32")]
impl From<Nonce> for JsNonce {
    fn from(nonce: Nonce) -> Self {
        JsNonce(nonce)
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl JsNonce {
    /// Generate a 96-bit, 12-byte nonce.
    /// This is the minimum nonce size typically recommended.
    ///
    /// # Arguments
    ///
    /// * `salt` - A salt. This may be left empty, but is recommended to avoid collision.
    pub fn generate_12(mut salt: Vec<u8>) -> JsNonce {
        Nonce::generate_12(&mut salt).into()
    }

    /// Generate a 128-bit, 16-byte nonce
    ///
    /// # Arguments
    ///
    /// * `salt` - A salt. This may be left empty, but is recommended to avoid collision.
    pub fn generate_16(mut salt: Vec<u8>) -> JsNonce {
        Nonce::generate_16(&mut salt).into()
    }

    /// Directly lift a 12-byte `Uint8Array` into a [`JsNonce`]
    ///
    /// # Arguments
    ///
    /// * `nonce` - The exact nonce to convert to a [`JsNonce`]
    pub fn from_uint8_array(arr: Box<[u8]>) -> JsNonce {
        Nonce::from(arr.to_vec()).into()
    }

    /// Expose the underlying bytes of a [`JsNonce`] as a 12-byte `Uint8Array`
    ///
    /// # Arguments
    ///
    /// * `self` - The [`JsNonce`] to convert to a `Uint8Array`
    pub fn to_uint8_array(&self) -> Box<[u8]> {
        match &self.0 {
            Nonce::Nonce12(nonce) => nonce.to_vec().into_boxed_slice(),
            Nonce::Nonce16(nonce) => nonce.to_vec().into_boxed_slice(),
            Nonce::Custom(nonce) => nonce.clone().into_boxed_slice(),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    // FIXME prop test with lots of inputs
    #[test]
    fn ipld_roundtrip_12() {
        let gen = Nonce::generate_12(&mut vec![]);
        let ipld = Ipld::from(gen.clone());

        let inner = if let Nonce::Nonce12(nonce) = gen {
            Ipld::Bytes(nonce.to_vec())
        } else {
            panic!("No conversion!")
        };

        assert_eq!(ipld, inner);
        assert_eq!(gen, ipld.try_into().unwrap());
    }

    // FIXME prop test with lots of inputs
    #[test]
    fn ipld_roundtrip_16() {
        let gen = Nonce::generate_16(&mut vec![]);
        let ipld = Ipld::from(gen.clone());

        let inner = if let Nonce::Nonce16(nonce) = gen {
            Ipld::Bytes(nonce.to_vec())
        } else {
            panic!("No conversion!")
        };

        assert_eq!(ipld, inner);
        assert_eq!(gen, ipld.try_into().unwrap());
    }

    // FIXME prop test with lots of inputs
    // #[test]
    // fn ser_de() {
    //     let gen = Nonce::generate_16(&mut vec![]);
    //     let ser = serde_json::to_string(&gen).unwrap();
    //     let de = serde_json::from_str(&ser).unwrap();

    //     assert_eq!(gen, de);
    // }
}
