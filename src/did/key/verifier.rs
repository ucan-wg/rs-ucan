use super::Signature;
use enum_as_inner::EnumAsInner;
use rsa::pkcs1::{DecodeRsaPublicKey, EncodeRsaPublicKey};
use std::{fmt::Display, str::FromStr};

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
        match self {
            Verifier::EdDsa(ed25519_pk) => write!(
                f,
                "did:key:z6Mk{}",
                bs58::encode(ed25519_pk.to_bytes()).into_string()
            ),
            Verifier::Es256k(secp256k1_pk) => write!(
                f,
                "did:key:zQ3s{}",
                bs58::encode(secp256k1_pk.to_sec1_bytes()).into_string()
            ),
            Verifier::P256(p256_key) => {
                write!(
                    f,
                    "did:key:zDn{}",
                    bs58::encode(p256_key.to_sec1_bytes()).into_string()
                )
            }
            Verifier::P384(p384_key) => write!(
                f,
                "did:key:z82{}",
                bs58::encode(p384_key.to_sec1_bytes()).into_string()
            ),
            Verifier::P521(p521_key) => write!(
                f,
                "did:key:z2J9{}",
                bs58::encode(p521_key.0.to_encoded_point(true).as_bytes()).into_string()
            ),
            Verifier::Rs256(rsa2048_key) => {
                write!(
                    f,
                    "did:key:z4MX{}",
                    bs58::encode(
                        rsa2048_key
                            .0
                            .to_pkcs1_der()
                            .expect("RSA key to encode") // FIXME?
                            .as_bytes()
                    )
                    .into_string()
                )
            }
            Verifier::Rs512(rsa4096_key) => write!(
                f,
                "did:key:zgg{}",
                bs58::encode(
                    rsa4096_key
                        .0
                        .to_pkcs1_der()
                        .expect("RSA key to encode") // FIXME?
                        .as_bytes()
                )
                .into_string()
            ),
            Verifier::BlsMinPk(bls_minpk_pk) => write!(
                f,
                "did:key:zUC7{}",
                bs58::encode(bls_minpk_pk.serialize()).into_string()
            ),
            Verifier::BlsMinSig(bls_minsig_pk) => write!(
                f,
                "did:key:zUC7{}",
                bs58::encode(bls_minsig_pk.serialize()).into_string()
            ),
        }
    }
}

impl FromStr for Verifier {
    type Err = String; // FIXME

    // FIXME needs tests
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() < 32 {
            // Smallest key size
            return Err("invalid did:key".to_string());
        }

        if let ("did:key:z", more) = s.split_at(9) {
            let bytes = more.as_bytes();
            match bytes.split_at(2) {
                ([0xed, _], _) => {
                    let vk = ed25519_dalek::VerifyingKey::try_from(&bytes[1..33])
                        .map_err(|e| e.to_string())?;

                    return Ok(Verifier::EdDsa(vk));
                }
                ([0xe7, _], _) => {
                    let vk = k256::ecdsa::VerifyingKey::from_sec1_bytes(&bytes[1..])
                        .map_err(|e| e.to_string())?;

                    return Ok(Verifier::Es256k(vk));
                }
                ([0x12, 0x00], key_bytes) => {
                    let vk = p256::ecdsa::VerifyingKey::from_sec1_bytes(key_bytes)
                        .map_err(|e| e.to_string())?;

                    return Ok(Verifier::P256(vk));
                }
                ([0x12, 0x01], key_bytes) => {
                    let vk = p384::ecdsa::VerifyingKey::from_sec1_bytes(key_bytes)
                        .map_err(|e| e.to_string())?;

                    return Ok(Verifier::P384(vk));
                }
                ([0x12, 0x05], key_bytes) => match key_bytes.len() {
                    2048 => {
                        let vk = rsa::pkcs1v15::VerifyingKey::from_pkcs1_der(key_bytes)
                            .map_err(|e| e.to_string())?;

                        return Ok(Verifier::Rs256(rs256::VerifyingKey(vk)));
                    }
                    4096 => {
                        let vk = rsa::pkcs1v15::VerifyingKey::from_pkcs1_der(key_bytes)
                            .map_err(|e| e.to_string())?;

                        return Ok(Verifier::Rs512(rs512::VerifyingKey(vk)));
                    }
                    _ => todo!(),
                },
                ([0xeb, 0x01], pk_bytes) => match pk_bytes.len() {
                    48 => {
                        let pk = blst::min_pk::PublicKey::deserialize(pk_bytes)
                            .map_err(|_| "Failed BLS MinPk deserialization")?;

                        return Ok(Verifier::BlsMinPk(pk));
                    }
                    96 => {
                        let pk = blst::min_sig::PublicKey::deserialize(pk_bytes)
                            .map_err(|_| "Failed BLS MinSig deserialization")?;

                        return Ok(Verifier::BlsMinSig(pk));
                    }
                    _ => return Err("invalid did:key".to_string()),
                },
                _ => {
                    return Err("invalid did:key".to_string());
                }
            }
        } else {
            return Err("invalid did:key".to_string());
        }
    }
}

#[cfg(feature = "test_utils")]
impl Arbitrary for Verifier {
    type Parameters = ();
    type Strategy = BoxedStrategy<Self>;

    fn arbitrary_with(_args: Self::Parameters) -> Self::Strategy {
        // NOTE these are just the test vectors from `did:key` v0.7
        prop_oneof![
             // did:key
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
         ].prop_map(|s: &str| Verifier::from_str(s).expect("did:key spec test vectors to work")).boxed()
    }
}
