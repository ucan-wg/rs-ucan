//! WebCrypto-compatible signature types and verifiers.

#[cfg(feature = "web_crypto")]
use crate::signature::ecdsa;

/// The WebCrypto-compatible signature types.
#[cfg(feature = "web_crypto")]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WebCrypto {
    /// 2048-bit RSA signature type
    Rs256_2048(rsa::Rs256<2048>),

    /// 4096-bit RSA signature type
    Rs256_4096(rsa::Rs256<4096>),

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
    /// Verifier for 2048-bit RSA signature type
    Rs256_2048(rsa::Rs256<2048>),

    /// Verifier for 4096-bit RSA signature type
    Rs256_4096(rsa::Rs256<4096>),

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
            WebCrypto::Rs256_2048(rs256) => rs256.prefix(),
            WebCrypto::Rs256_4096(rs512) => r512.prefix(),
            WebCrypto::Es256(es256) => es256.prefix(),
            WebCrypto::Es384(es384) => es384.prefix(),
            WebCrypto::Es512(es512) => es512.prefix(),
            WebCrypto::Ed25519(ed25519) => ed25519.prefix(),
        }
    }

    fn config_tags(&self) -> Vec<u64> {
        match self {
            WebCrypto::Rs256_2048(rs256) => rs256.config_tags(),
            WebCrypto::Rs256_4096(rs512) => rs512.config_tags(),
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
            0x1205 => {
                if bytes.len() < 3 {
                    return None;
                }
                match bytes[1..=2] {
                    [0x12, 0x0100] => Some((
                        WebCrypto::Rs256_2048(rsa::Rs256::<2048>::default()),
                        &bytes[3..],
                    )),
                    [0x12, 0x0200] => Some((
                        WebCrypto::Rs256_4096(rsa::Rs256::<4096>::default()),
                        &bytes[3..],
                    )),
                    _ => None,
                }
            }
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
