use did_url::DID;
use enum_as_inner::EnumAsInner;
use libipld_core::ipld::Ipld;
use serde::{Deserialize, Serialize};
use std::{fmt, string::ToString};
use thiserror::Error;

#[cfg(feature = "test_utils")]
use proptest::prelude::*;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
/// A [Decentralized Identifier (DID)][wiki]
///
/// This is a newtype wrapper around the [`DID`] type from the [`did_url`] crate.
///
/// # Examples
///
/// ```rust
/// # use ucan::did;
/// #
/// let did = did::Newtype::try_from("did:example:123".to_string()).unwrap();
/// assert_eq!(did.0.method(), "example");
/// ```
///
/// [wiki]: https://en.wikipedia.org/wiki/Decentralized_identifier
pub struct Newtype(pub DID);

impl Serialize for Newtype {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        String::from(self.clone()).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for Newtype {
    fn deserialize<D>(deserializer: D) -> Result<Newtype, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let string = String::deserialize(deserializer)?;
        Newtype::try_from(string).map_err(serde::de::Error::custom)
    }
}

impl From<Newtype> for String {
    fn from(did: Newtype) -> Self {
        did.0.to_string()
    }
}

impl TryFrom<String> for Newtype {
    type Error = <DID as TryFrom<String>>::Error;

    fn try_from(string: String) -> Result<Self, Self::Error> {
        DID::parse(&string).map(Newtype)
    }
}

impl fmt::Display for Newtype {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0.to_string())
    }
}

impl From<Newtype> for Ipld {
    fn from(did: Newtype) -> Self {
        did.into()
    }
}

impl TryFrom<Ipld> for Newtype {
    type Error = FromIpldError;

    fn try_from(ipld: Ipld) -> Result<Self, Self::Error> {
        match ipld {
            Ipld::String(string) => {
                Newtype::try_from(string).map_err(FromIpldError::StructuralError)
            }
            other => Err(FromIpldError::NotAnIpldString(other)),
        }
    }
}

/// Errors that can occur when converting to or from a [`Newtype`]
#[derive(Debug, Clone, EnumAsInner, PartialEq, Error)]
pub enum FromIpldError {
    /// Strutural errors in the [`Newtype`]
    #[error(transparent)]
    StructuralError(#[from] did_url::Error),

    /// The [`Ipld`] was not a string
    #[error("Not an IPLD String")]
    NotAnIpldString(Ipld),
}

impl Serialize for FromIpldError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        self.to_string().serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for FromIpldError {
    fn deserialize<D>(deserializer: D) -> Result<FromIpldError, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let ipld = Ipld::deserialize(deserializer)?;
        Ok(FromIpldError::NotAnIpldString(ipld))
    }
}

#[cfg(feature = "test_utils")]
impl Arbitrary for Newtype {
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
          ].prop_map(|s: &str| Newtype(did_url::DID::parse(s).expect("did:key spec test vectors to work"))).boxed()
    }
}
