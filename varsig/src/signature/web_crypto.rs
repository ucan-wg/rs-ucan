//! WebCrypto-compatible signature types and verifiers.

#[cfg(feature = "web_crypto")]
use crate::signature::ecdsa;

/// The WebCrypto-compatible signature types.
#[cfg(feature = "web_crypto")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WebCrypto {
    /// ES256 signature type
    Es256(ecdsa::Es256),

    /// ES384 signature type
    Es384(ecdsa::Es384),

    /// ES512 signature type
    Es512(ecdsa::Es512),

    /// Ed25519 signature type
    Ed25519(eddsa::Ed25519),
}

#[cfg(feature = "web_crypto")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WebCryptoVerifier {
    /// Verifier for ES256 signature type
    Es256(ecdsa::Es256),

    /// Verifier for ES384 signature type
    Es384(ecdsa::Es384),

    /// Verifier for ES512 signature type
    Es512(ecdsa::Es512),

    /// Verifier for Ed25519 signature type
    Ed25519(eddsa::Ed25519),
}

#[cfg(feature = "web_crypto")]
impl Verify for WebCrypto {
    type Signature = WebCrypto;
    type Verifier = WebCryptoVerifier;

    fn prefix(&self) -> u64 {
        match self {
            WebCrypto::Es256(es256) => es256.prefix(),
            WebCrypto::Es384(es384) => es384.prefix(),
            WebCrypto::Es512(es512) => es512.prefix(),
            WebCrypto::Ed25519(ed25519) => ed25519.prefix(),
        }
    }

    fn config_tags(&self) -> Vec<u64> {
        match self {
            WebCrypto::Es256(es256) => es256.config_tags(),
            WebCrypto::Es384(es384) => es384.config_tags(),
            WebCrypto::Es512(es512) => es512.config_tags(),
            WebCrypto::Ed25519(ed25519) => ed25519.config_tags(),
        }
    }

    fn try_from_tags(bytes: &[u64]) -> Option<(Self, &[u64])> {
        if bytes.is_empty() {
            return None;
        }

        match bytes[0] {
            0xec => {
                if bytes.len() < 3 {
                    return None;
                }
                match bytes[1..=2] {
                    [0x1201, 0x15] => {
                        Some((WebCrypto::Es256(ecdsa::Es256::default()), &bytes[3..]))
                    }
                    [0x1201, 0x20] => {
                        Some((WebCrypto::Es384(ecdsa::Es384::default()), &bytes[3..]))
                    }
                    [0x1201, 0x25] => {
                        Some((WebCrypto::Es512(ecdsa::Es512::default()), &bytes[3..]))
                    }
                    _ => None,
                }
            }
            0xed => {
                if bytes.len() < 3 || bytes[1..=2] != [0xed, 0x13] {
                    return None;
                }
                Some((WebCrypto::Ed25519(eddsa::Ed25519::default()), &bytes[3..]))
            }
            _ => None,
        }
    }
}
