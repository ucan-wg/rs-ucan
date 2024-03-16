use super::Signature;
use blst::BLST_ERROR;
use did_url::DID;
use enum_as_inner::EnumAsInner;
use libipld_core::ipld::Ipld;
use multibase;
use multibase::Base;
use rsa::pkcs1::{DecodeRsaPublicKey, EncodeRsaPublicKey};
use serde::{Deserialize, Serialize};
use signature as sig;
use std::{fmt::Display, str::FromStr};
use thiserror::Error;

#[cfg(feature = "test_utils")]
use proptest::prelude::*;

#[cfg(feature = "eddsa")]
use ed25519_dalek;

#[cfg(feature = "es256")]
use p256;

#[cfg(feature = "es256k")]
use k256;

#[cfg(feature = "es384")]
use p384;

#[cfg(feature = "es512")]
use crate::crypto::es512;

#[cfg(feature = "rs256")]
use crate::crypto::rs256;

#[cfg(feature = "rs512")]
use crate::crypto::rs512;

#[cfg(feature = "bls")]
use blst;

/// Verifiers (public/verifying keys) for `did:key`.
#[derive(Debug, Clone, PartialEq, Eq, EnumAsInner)]
pub enum Verifier {
    /// `EdDSA` verifying key.
    #[cfg(feature = "eddsa")]
    EdDsa(ed25519_dalek::VerifyingKey),

    /// `ES256K` (`secp256k1`) verifying key.
    #[cfg(feature = "es256k")]
    Es256k(k256::ecdsa::VerifyingKey),

    /// `P-256` verifying key.
    #[cfg(feature = "es256")]
    P256(p256::ecdsa::VerifyingKey),

    /// `P-384` verifying key.
    #[cfg(feature = "es384")]
    P384(p384::ecdsa::VerifyingKey),

    /// `P-521` verifying key.
    #[cfg(feature = "es512")]
    P521(es512::VerifyingKey),

    /// `RS256` verifying key.
    #[cfg(feature = "rs256")]
    Rs256(rs256::VerifyingKey),

    /// `RS512` verifying key.
    #[cfg(feature = "rs512")]
    Rs512(rs512::VerifyingKey),

    /// `BLS 12-381` verifying key for the "min pub key" variant.
    #[cfg(feature = "bls")]
    BlsMinPk(blst::min_pk::PublicKey),

    /// `BLS 12-381` verifying key for the "min sig" variant.
    #[cfg(feature = "bls")]
    BlsMinSig(blst::min_sig::PublicKey),
}

impl PartialOrd for Verifier {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.to_string().partial_cmp(&other.to_string())
    }
}

impl Ord for Verifier {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.to_string().cmp(&other.to_string())
    }
}

impl signature::Verifier<Signature> for Verifier {
    fn verify(&self, msg: &[u8], signature: &Signature) -> Result<(), signature::Error> {
        match (self, signature) {
            (Verifier::EdDsa(vk), Signature::EdDsa(sig)) => {
                vk.verify(msg, sig).map_err(signature::Error::from_source)
            }
            (Verifier::Es256k(vk), Signature::Es256k(sig)) => {
                vk.verify(msg, sig).map_err(signature::Error::from_source)
            }
            (Verifier::P256(vk), Signature::P256(sig)) => {
                vk.verify(msg, sig).map_err(signature::Error::from_source)
            }
            (Verifier::P384(vk), Signature::P384(sig)) => {
                vk.verify(msg, sig).map_err(signature::Error::from_source)
            }
            (Verifier::P521(vk), Signature::P521(sig)) => {
                vk.verify(msg, sig).map_err(signature::Error::from_source)
            }
            (Verifier::Rs256(vk), Signature::Rs256(sig)) => {
                vk.verify(msg, sig).map_err(signature::Error::from_source)
            }
            (Verifier::Rs512(vk), Signature::Rs512(sig)) => {
                vk.verify(msg, sig).map_err(signature::Error::from_source)
            }
            (Verifier::BlsMinPk(vk), Signature::BlsMinPk(sig)) => {
                vk.verify(msg, sig).map_err(signature::Error::from_source)
            }
            (Verifier::BlsMinSig(vk), Signature::BlsMinSig(sig)) => {
                vk.verify(msg, sig).map_err(signature::Error::from_source)
            }
            (_, _) => Err(signature::Error::from_source(
                "invalid signature type for verifier",
            )),
        }
    }
}

impl Display for Verifier {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let inner = match self {
            Verifier::EdDsa(ed25519_pk) => {
                let mut buf = [0u8; 2];
                let tag = unsigned_varint::encode::u8(0xed, &mut buf);

                let mut payload: Vec<u8> = tag.to_vec();
                let bytes = ed25519_pk.to_bytes();
                payload.extend_from_slice(&bytes);

                multibase::encode(Base::Base58Btc, payload)
            }
            Verifier::Es256k(secp256k1_pk) => {
                let mut buf = [0u8; 2];
                let tag = unsigned_varint::encode::u8(0xe7, &mut buf);

                let mut payload = tag.to_vec();
                let bytes = secp256k1_pk.to_sec1_bytes();
                payload.extend_from_slice(&bytes);

                multibase::encode(Base::Base58Btc, payload)
            }
            Verifier::P256(p256_key) => {
                let mut buf = [0u8; 3];
                let tag = unsigned_varint::encode::u16(0x1200, &mut buf);

                let mut payload = tag.to_vec();
                let point = p256_key.to_encoded_point(true);
                payload.extend_from_slice(point.as_bytes());

                multibase::encode(Base::Base58Btc, payload)
            }
            Verifier::P384(p384_key) => {
                let mut buf = [0u8; 3];
                let tag = unsigned_varint::encode::u16(0x1201, &mut buf);

                let mut payload = tag.to_vec();
                let point = p384_key.to_encoded_point(true);
                payload.extend_from_slice(point.as_bytes());

                multibase::encode(Base::Base58Btc, payload)
            }
            Verifier::P521(p521_key) => {
                let mut buf = [0u8; 3];
                let tag = unsigned_varint::encode::u16(0x1202, &mut buf);

                let mut payload = tag.to_vec();
                let point = p521_key.0.to_encoded_point(true);
                payload.extend_from_slice(point.as_bytes());

                multibase::encode(Base::Base58Btc, payload)
            }
            Verifier::Rs256(rsa2048_key) => {
                let mut buf = [0u8; 3];
                let tag = unsigned_varint::encode::u16(0x1205, &mut buf);

                let mut payload = tag.to_vec();
                let raw = rsa2048_key.0.to_pkcs1_der().map_err(|_| std::fmt::Error)?; // NOTE: technically should never fail
                payload.extend_from_slice(raw.as_bytes());

                multibase::encode(Base::Base58Btc, payload)
            }
            Verifier::Rs512(rsa4096_key) => {
                let mut buf = [0u8; 3];
                let tag = unsigned_varint::encode::u16(0x1205, &mut buf);

                let mut payload = tag.to_vec();
                let raw = rsa4096_key.0.to_pkcs1_der().map_err(|_| std::fmt::Error)?; // NOTE: technically should never fail
                payload.extend_from_slice(raw.as_bytes());

                multibase::encode(Base::Base58Btc, payload)
            }
            Verifier::BlsMinPk(bls_minpk_pk) => {
                let bytes = bls_minpk_pk.compress();

                let mut buf = [0u8; 2];
                let tag = unsigned_varint::encode::u8(0xeb, &mut buf);

                let mut payload = tag.to_vec();
                payload.extend_from_slice(&bytes);

                multibase::encode(Base::Base58Btc, payload)
            }
            Verifier::BlsMinSig(bls_minsig_pk) => {
                let bytes = bls_minsig_pk.compress();

                let mut buf = [0u8; 2];
                let tag = unsigned_varint::encode::u8(0xeb, &mut buf);

                let mut payload = tag.to_vec();
                payload.extend_from_slice(&bytes);

                multibase::encode(Base::Base58Btc, payload)
            }
        };

        write!(f, "did:key:{}", inner)
    }
}

impl FromStr for Verifier {
    type Err = FromStrError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 32 {
            // Smallest key size
            return Err(FromStrError::TooShort);
        }

        match s.split_at(8) {
            ("did:key:", more) => {
                let (_base, varint_bytes): (multibase::Base, Vec<u8>) = multibase::decode(more)?;
                let (tag, rest) = unsigned_varint::decode::u16(&varint_bytes)?;

                // FIXME also check max length on bytes
                match tag {
                    0xed => {
                        let arr: [u8; 32] = rest.try_into().map_err(|_| FromStrError::TooShort)?;

                        let vk = ed25519_dalek::VerifyingKey::from_bytes(&arr)
                            .map_err(FromStrError::CannotParseEdDsa)?;

                        Ok(Verifier::EdDsa(vk))
                    }
                    0xe7 => {
                        let vk = k256::ecdsa::VerifyingKey::from_sec1_bytes(&rest)
                            .map_err(FromStrError::CannotParseEs256k)?;

                        Ok(Verifier::Es256k(vk))
                    }
                    0x1200 => {
                        let vk = p256::ecdsa::VerifyingKey::from_sec1_bytes(rest)
                            .map_err(FromStrError::CannotParseP256)?;

                        Ok(Verifier::P256(vk))
                    }
                    0x1201 => {
                        let vk = p384::ecdsa::VerifyingKey::from_sec1_bytes(rest)
                            .map_err(FromStrError::CannotParseP384)?;

                        Ok(Verifier::P384(vk))
                    }
                    0x1202 => {
                        let vk = p521::ecdsa::VerifyingKey::from_sec1_bytes(rest)
                            .map_err(FromStrError::CannotParseP521)?;

                        Ok(Verifier::P521(es512::VerifyingKey(vk)))
                    }
                    0x1205 => match rest.len() {
                        // 256-bytes plus params
                        270 => {
                            let vk = rsa::pkcs1v15::VerifyingKey::from_pkcs1_der(rest)
                                .map_err(FromStrError::CannotParseRs256)?;

                            Ok(Verifier::Rs256(rs256::VerifyingKey(vk)))
                        }
                        // 512-bytes plus params
                        526 => {
                            let vk = rsa::pkcs1v15::VerifyingKey::from_pkcs1_der(rest)
                                .map_err(FromStrError::CannotParseRs512)?;

                            Ok(Verifier::Rs512(rs512::VerifyingKey(vk)))
                        }
                        len => Err(FromStrError::InvalidRsaLength(len)),
                    },
                    0xeb => match rest.len() {
                        48 => {
                            let pk = blst::min_pk::PublicKey::deserialize(rest)
                                .map_err(FromStrError::CannotParseBlsMinPk)?;

                            Ok(Verifier::BlsMinPk(pk))
                        }
                        96 => {
                            let pk = blst::min_sig::PublicKey::deserialize(rest)
                                .map_err(FromStrError::CannotParseBlsMinSig)?;

                            Ok(Verifier::BlsMinSig(pk))
                        }
                        len => Err(FromStrError::InvalidBlsLength(len)),
                    },
                    word => Err(FromStrError::UnexpectedPrefix(word)),
                }
            }

            (s, _) => Err(FromStrError::UnexpectedHeader(s.to_string())),
        }
    }
}

impl From<Verifier> for Ipld {
    fn from(v: Verifier) -> Self {
        v.to_string().into()
    }
}

impl TryFrom<Ipld> for Verifier {
    type Error = (); // FIXME

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        if let Ipld::String(s) = ipld {
            Verifier::from_str(&s).map_err(|_| ())
        } else {
            Err(())
        }
    }
}

#[derive(Debug, Error)]
pub enum FromStrError {
    #[error("not a did:key prefix: {0}")]
    NotADidKey(usize),

    #[error("unexpected prefix: {0:?}")]
    UnexpectedPrefix(u16),

    #[error("unexpected header: {0}")]
    UnexpectedHeader(String),

    #[error("unexpected BLS length: {0}")]
    InvalidBlsLength(usize),

    #[error("Invalid RSA length: {0}")]
    InvalidRsaLength(usize),

    #[error("key too short")]
    TooShort,

    #[error("cannot parse EdDSA key: {0}")]
    CannotParseEdDsa(sig::Error),

    #[error("cannot parse ES256K key: {0}")]
    CannotParseEs256k(sig::Error),

    #[error("cannot parse P-256 key: {0}")]
    CannotParseP256(sig::Error),

    #[error("cannot parse P-384 key: {0}")]
    CannotParseP384(sig::Error),

    #[error("cannot parse P-521 key: {0}")]
    CannotParseP521(sig::Error),

    #[error("cannot parse RS256 key: {0}")]
    CannotParseRs256(rsa::pkcs1::Error),

    #[error("cannot parse RS512 key: {0}")]
    CannotParseRs512(rsa::pkcs1::Error),

    #[error("cannot parse BLS min pk key: {0:?}")]
    CannotParseBlsMinPk(BLST_ERROR),

    #[error("cannot parse BLS min sig key: {0:?}")]
    CannotParseBlsMinSig(BLST_ERROR),

    #[error("cannot decode multibase: {0}")]
    CannotDecodeMultibase(#[from] multibase::Error),

    #[error("cannot parse tag: {0}")]
    CannotParseTag(#[from] unsigned_varint::decode::Error),
}

impl PartialEq for FromStrError {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (FromStrError::NotADidKey(a), FromStrError::NotADidKey(b)) => a == b,
            (FromStrError::UnexpectedPrefix(a), FromStrError::UnexpectedPrefix(b)) => a == b,
            (FromStrError::TooShort, FromStrError::TooShort) => true,
            (FromStrError::CannotParseEdDsa(a), FromStrError::CannotParseEdDsa(b)) => {
                a.to_string() == b.to_string()
            }
            (FromStrError::CannotParseEs256k(a), FromStrError::CannotParseEs256k(b)) => {
                a.to_string() == b.to_string()
            }
            (FromStrError::CannotParseP256(a), FromStrError::CannotParseP256(b)) => {
                a.to_string() == b.to_string()
            }
            (FromStrError::CannotParseP384(a), FromStrError::CannotParseP384(b)) => {
                a.to_string() == b.to_string()
            }
            (FromStrError::CannotParseP521(a), FromStrError::CannotParseP521(b)) => {
                a.to_string() == b.to_string()
            }
            (FromStrError::CannotParseRs256(a), FromStrError::CannotParseRs256(b)) => {
                a.to_string() == b.to_string()
            }
            (FromStrError::CannotParseRs512(a), FromStrError::CannotParseRs512(b)) => {
                a.to_string() == b.to_string()
            }
            (FromStrError::CannotParseBlsMinPk(a), FromStrError::CannotParseBlsMinPk(b)) => a == b,
            (FromStrError::CannotParseBlsMinSig(a), FromStrError::CannotParseBlsMinSig(b)) => {
                a == b
            }
            (
                FromStrError::CannotDecodeMultibase(lhs),
                FromStrError::CannotDecodeMultibase(rhs),
            ) => lhs == rhs,
            (FromStrError::CannotParseTag(lhs), FromStrError::CannotParseTag(rhs)) => lhs == rhs,
            _ => false,
        }
    }
}

impl Serialize for Verifier {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Verifier {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = String::deserialize(deserializer)?;
        Verifier::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl From<Verifier> for DID {
    fn from(v: Verifier) -> Self {
        DID::parse(&v.to_string()).expect("verifier to be a valid DID")
    }
}

impl TryFrom<DID> for Verifier {
    type Error = FromStrError;

    fn try_from(did: DID) -> Result<Self, Self::Error> {
        Verifier::from_str(&did.to_string())
    }
}

#[cfg(feature = "test_utils")]
impl Arbitrary for Verifier {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        // NOTE these are just the test vectors from `did:key` v0.7
        prop_oneof![
            // ed25519
            Just("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK"),

            // secp256k1
            Just("did:key:zQ3shokFTS3brHcDQrn82RUDfCZESWL1ZdCEJwekUDPQiYBme"),
            Just("did:key:zQ3shtxV1FrJfhqE1dvxYRcCknWNjHc3c5X1y3ZSoPDi2aur2"),
            Just("did:key:zQ3shZc2QzApp2oymGvQbzP8eKheVshBHbU4ZYjeXqwSKEn6N"),

            // BLS
            Just("did:key:zUC7K4ndUaGZgV7Cp2yJy6JtMoUHY6u7tkcSYUvPrEidqBmLCTLmi6d5WvwnUqejscAkERJ3bfjEiSYtdPkRSE8kSa11hFBr4sTgnbZ95SJj19PN2jdvJjyzpSZgxkyyxNnBNnY"),
            Just("did:key:zUC7KKoJk5ttwuuc8pmQDiUmtckEPTwcaFVZe4DSFV7fURuoRnD17D3xkBK3A9tZqdADkTTMKSwNkhjo9Hs6HfgNUXo48TNRaxU6XPLSPdRgMc15jCD5DfN34ixjoVemY62JxnW"),

            // P-256
            Just("did:key:zDnaerDaTF5BXEavCrfRZEk316dpbLsfPDZ3WJ5hRTPFU2169"),
            Just("did:key:zDnaerx9CtbPJ1q36T5Ln5wYt3MQYeGRG5ehnPAmxcf5mDZpv"),

            // P-384
            Just("did:key:z82Lm1MpAkeJcix9K8TMiLd5NMAhnwkjjCBeWHXyu3U4oT2MVJJKXkcVBgjGhnLBn2Kaau9"),
            Just("did:key:z82LkvCwHNreneWpsgPEbV3gu1C6NFJEBg4srfJ5gdxEsMGRJUz2sG9FE42shbn2xkZJh54"),

            // P-521
            Just("did:key:z2J9gaYxrKVpdoG9A4gRnmpnRCcxU6agDtFVVBVdn1JedouoZN7SzcyREXXzWgt3gGiwpoHq7K68X4m32D8HgzG8wv3sY5j7"),
            Just("did:key:z2J9gcGdb2nEyMDmzQYv2QZQcM1vXktvy1Pw4MduSWxGabLZ9XESSWLQgbuPhwnXN7zP7HpTzWqrMTzaY5zWe6hpzJ2jnw4f"),

            // RSA-2048
            Just("did:key:z4MXj1wBzi9jUstyPMS4jQqB6KdJaiatPkAtVtGc6bQEQEEsKTic4G7Rou3iBf9vPmT5dbkm9qsZsuVNjq8HCuW1w24nhBFGkRE4cd2Uf2tfrB3N7h4mnyPp1BF3ZttHTYv3DLUPi1zMdkULiow3M1GfXkoC6DoxDUm1jmN6GBj22SjVsr6dxezRVQc7aj9TxE7JLbMH1wh5X3kA58H3DFW8rnYMakFGbca5CB2Jf6CnGQZmL7o5uJAdTwXfy2iiiyPxXEGerMhHwhjTA1mKYobyk2CpeEcmvynADfNZ5MBvcCS7m3XkFCMNUYBS9NQ3fze6vMSUPsNa6GVYmKx2x6JrdEjCk3qRMMmyjnjCMfR4pXbRMZa3i"),

            // RSA-4096
            Just("did:key:zgghBUVkqmWS8e1ioRVp2WN9Vw6x4NvnE9PGAyQsPqM3fnfPf8EdauiRVfBTcVDyzhqM5FFC7ekAvuV1cJHawtfgB9wDcru1hPDobk3hqyedijhgWmsYfJCmodkiiFnjNWATE7PvqTyoCjcmrc8yMRXmFPnoASyT5beUd4YZxTE9VfgmavcPy3BSouNmASMQ8xUXeiRwjb7xBaVTiDRjkmyPD7NYZdXuS93gFhyDFr5b3XLg7Rfj9nHEqtHDa7NmAX7iwDAbMUFEfiDEf9hrqZmpAYJracAjTTR8Cvn6mnDXMLwayNG8dcsXFodxok2qksYF4D8ffUxMRmyyQVQhhhmdSi4YaMPqTnC1J6HTG9Yfb98yGSVaWi4TApUhLXFow2ZvB6vqckCNhjCRL2R4MDUSk71qzxWHgezKyDeyThJgdxydrn1osqH94oSeA346eipkJvKqYREXBKwgB5VL6WF4qAK6sVZxJp2dQBfCPVZ4EbsBQaJXaVK7cNcWG8tZBFWZ79gG9Cu6C4u8yjBS8Ux6dCcJPUTLtixQu4z2n5dCsVSNdnP1EEs8ZerZo5pBgc68w4Yuf9KL3xVxPnAB1nRCBfs9cMU6oL1EdyHbqrTfnjE8HpY164akBqe92LFVsk8RusaGsVPrMekT8emTq5y8v8CabuZg5rDs3f9NPEtogjyx49wiub1FecM5B7QqEcZSYiKHgF4mfkteT2")
        ]
        .prop_map(|s: &str| Verifier::from_str(s).expect("did:key spec test vectors to work"))
        .boxed()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use pretty_assertions as pretty;
    use testresult::TestResult;

    mod serialization {
        use super::*;

        fn roundtrip(s: &str) -> TestResult {
            let v = Verifier::from_str(s)?;
            let serialized = v.to_string();
            pretty::assert_eq!(s, serialized);
            Ok(())
        }

        #[test_log::test]
        fn test_ed25519_parse() -> TestResult {
            roundtrip("did:key:z6MkhaXgBZDvotDkL5257faiztiGiC2QtKLGpbnnEGta2doK")
        }

        #[test_log::test]
        fn test_secp256k_1_parse() -> TestResult {
            roundtrip("did:key:zQ3shokFTS3brHcDQrn82RUDfCZESWL1ZdCEJwekUDPQiYBme")
        }

        #[test_log::test]
        fn test_secp256k_2_parse() -> TestResult {
            roundtrip("did:key:zQ3shtxV1FrJfhqE1dvxYRcCknWNjHc3c5X1y3ZSoPDi2aur2")
        }

        #[test_log::test]
        fn test_secp256k_3_parse() -> TestResult {
            roundtrip("did:key:zQ3shZc2QzApp2oymGvQbzP8eKheVshBHbU4ZYjeXqwSKEn6N")
        }

        #[test_log::test]
        fn test_bls_1_parse() -> TestResult {
            roundtrip("did:key:zUC7K4ndUaGZgV7Cp2yJy6JtMoUHY6u7tkcSYUvPrEidqBmLCTLmi6d5WvwnUqejscAkERJ3bfjEiSYtdPkRSE8kSa11hFBr4sTgnbZ95SJj19PN2jdvJjyzpSZgxkyyxNnBNnY")
        }

        #[test_log::test]
        fn test_bls_2_parse() -> TestResult {
            roundtrip("did:key:zUC7KKoJk5ttwuuc8pmQDiUmtckEPTwcaFVZe4DSFV7fURuoRnD17D3xkBK3A9tZqdADkTTMKSwNkhjo9Hs6HfgNUXo48TNRaxU6XPLSPdRgMc15jCD5DfN34ixjoVemY62JxnW")
        }

        #[test_log::test]
        fn test_p256_1_parse() -> TestResult {
            roundtrip("did:key:zDnaerDaTF5BXEavCrfRZEk316dpbLsfPDZ3WJ5hRTPFU2169")
        }

        #[test_log::test]
        fn test_p256_2_parse() -> TestResult {
            roundtrip("did:key:zDnaerx9CtbPJ1q36T5Ln5wYt3MQYeGRG5ehnPAmxcf5mDZpv")
        }

        #[test_log::test]
        fn test_p384_1_parse() -> TestResult {
            roundtrip(
                "did:key:z82Lm1MpAkeJcix9K8TMiLd5NMAhnwkjjCBeWHXyu3U4oT2MVJJKXkcVBgjGhnLBn2Kaau9",
            )
        }

        #[test_log::test]
        fn test_p384_2_parse() -> TestResult {
            roundtrip(
                "did:key:z82LkvCwHNreneWpsgPEbV3gu1C6NFJEBg4srfJ5gdxEsMGRJUz2sG9FE42shbn2xkZJh54",
            )
        }

        #[test_log::test]
        fn test_p521_1_parse() -> TestResult {
            roundtrip("did:key:z2J9gaYxrKVpdoG9A4gRnmpnRCcxU6agDtFVVBVdn1JedouoZN7SzcyREXXzWgt3gGiwpoHq7K68X4m32D8HgzG8wv3sY5j7")
        }

        #[test_log::test]
        fn test_p521_2_parse() -> TestResult {
            roundtrip("did:key:z2J9gcGdb2nEyMDmzQYv2QZQcM1vXktvy1Pw4MduSWxGabLZ9XESSWLQgbuPhwnXN7zP7HpTzWqrMTzaY5zWe6hpzJ2jnw4f")
        }

        #[test_log::test]
        fn test_rs256_parse() -> TestResult {
            roundtrip("did:key:z4MXj1wBzi9jUstyPMS4jQqB6KdJaiatPkAtVtGc6bQEQEEsKTic4G7Rou3iBf9vPmT5dbkm9qsZsuVNjq8HCuW1w24nhBFGkRE4cd2Uf2tfrB3N7h4mnyPp1BF3ZttHTYv3DLUPi1zMdkULiow3M1GfXkoC6DoxDUm1jmN6GBj22SjVsr6dxezRVQc7aj9TxE7JLbMH1wh5X3kA58H3DFW8rnYMakFGbca5CB2Jf6CnGQZmL7o5uJAdTwXfy2iiiyPxXEGerMhHwhjTA1mKYobyk2CpeEcmvynADfNZ5MBvcCS7m3XkFCMNUYBS9NQ3fze6vMSUPsNa6GVYmKx2x6JrdEjCk3qRMMmyjnjCMfR4pXbRMZa3i")
        }

        #[test_log::test]
        fn test_rs512_parse() -> TestResult {
            roundtrip("did:key:zgghBUVkqmWS8e1ioRVp2WN9Vw6x4NvnE9PGAyQsPqM3fnfPf8EdauiRVfBTcVDyzhqM5FFC7ekAvuV1cJHawtfgB9wDcru1hPDobk3hqyedijhgWmsYfJCmodkiiFnjNWATE7PvqTyoCjcmrc8yMRXmFPnoASyT5beUd4YZxTE9VfgmavcPy3BSouNmASMQ8xUXeiRwjb7xBaVTiDRjkmyPD7NYZdXuS93gFhyDFr5b3XLg7Rfj9nHEqtHDa7NmAX7iwDAbMUFEfiDEf9hrqZmpAYJracAjTTR8Cvn6mnDXMLwayNG8dcsXFodxok2qksYF4D8ffUxMRmyyQVQhhhmdSi4YaMPqTnC1J6HTG9Yfb98yGSVaWi4TApUhLXFow2ZvB6vqckCNhjCRL2R4MDUSk71qzxWHgezKyDeyThJgdxydrn1osqH94oSeA346eipkJvKqYREXBKwgB5VL6WF4qAK6sVZxJp2dQBfCPVZ4EbsBQaJXaVK7cNcWG8tZBFWZ79gG9Cu6C4u8yjBS8Ux6dCcJPUTLtixQu4z2n5dCsVSNdnP1EEs8ZerZo5pBgc68w4Yuf9KL3xVxPnAB1nRCBfs9cMU6oL1EdyHbqrTfnjE8HpY164akBqe92LFVsk8RusaGsVPrMekT8emTq5y8v8CabuZg5rDs3f9NPEtogjyx49wiub1FecM5B7QqEcZSYiKHgF4mfkteT2")
        }
    }
}
