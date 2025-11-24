//! Varsig header

use crate::{
    codec::Codec,
    signer::{AsyncSign, Sign, SignerError},
    verify::Verify,
};
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;

#[cfg(feature = "dag_cbor")]
use serde_ipld_dagcbor::codec::DagCborCodec;

#[cfg(feature = "dag_json")]
use serde_ipld_dagjson::codec::DagJsonCodec;

/// Top-level Varsig header type.
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd, Ord, Eq, Hash)]
pub struct Varsig<V: Verify, C: Codec<T>, T> {
    verifier_cfg: V,
    codec: C,
    _data: PhantomData<T>,
}

impl<V: Verify, C: Codec<T>, T> Varsig<V, C, T> {
    /// Create a new Varsig header.
    ///
    /// ## Parameters
    ///
    /// - `verifier`: The verifier to use for the Varsig header.
    /// - `codec`: The codec to use for encoding the payload.
    pub const fn new(verifier_cfg: V, codec: C) -> Self {
        Varsig {
            verifier_cfg,
            codec,
            _data: PhantomData,
        }
    }

    /// Get the verifier for this Varsig header.
    pub const fn verifier_cfg(&self) -> &V {
        &self.verifier_cfg
    }

    /// Get the codec for this Varsig header.
    pub const fn codec(&self) -> &C {
        &self.codec
    }

    /// Try to synchronously sign a payload with the provided signing key.
    ///
    /// # Errors
    ///
    /// If signing fails, a `SignerError` is returned.
    #[allow(clippy::type_complexity)]
    pub fn try_sign(
        &self,
        sk: &V::Signer,
        payload: &T,
    ) -> Result<(V::Signature, Vec<u8>), SignerError<C::EncodingError, V::SignError>>
    where
        V: Sign,
        C: Codec<T>,
        T: Serialize,
    {
        self.verifier_cfg.try_sign(&self.codec, sk, payload)
    }

    /// Try to asynchronously sign a payload with the provided signing key.
    ///
    /// # Errors
    ///
    /// If encoding or signing fails, a `SignerError` is returned.
    pub async fn try_sign_async(
        &self,
        sk: &V::AsyncSigner,
        payload: &T,
    ) -> Result<(V::Signature, Vec<u8>), SignerError<C::EncodingError, V::AsyncSignError>>
    where
        V: AsyncSign,
        C: Codec<T>,
        T: Serialize,
    {
        self.verifier_cfg
            .try_sign_async(&self.codec, sk, payload)
            .await
    }

    /// Try to verify a signature for some payload.
    ///
    /// # Errors
    ///
    /// If encoding or signature verification fails, a `VerificationError` is returned.
    pub fn try_verify(
        &self,
        verifier: &V::Verifier,
        payload: &T,
        signature: &V::Signature,
    ) -> Result<(), crate::verify::VerificationError<C::EncodingError>> {
        self.verifier_cfg()
            .try_verify(&self.codec, verifier, signature, payload)
    }
}

#[cfg(feature = "dag_cbor")]
impl<V: Verify + Default, T> Default for Varsig<V, DagCborCodec, T>
where
    DagCborCodec: Codec<T>,
{
    fn default() -> Self {
        Varsig {
            verifier_cfg: V::default(),
            codec: DagCborCodec,
            _data: PhantomData,
        }
    }
}

#[cfg(feature = "dag_json")]
impl<V: Verify + Default, T> Default for Varsig<V, DagJsonCodec, T>
where
    DagJsonCodec: Codec<T>,
{
    fn default() -> Self {
        Varsig {
            verifier_cfg: V::default(),
            codec: DagJsonCodec,
            _data: PhantomData,
        }
    }
}

impl<V: Verify, C: Codec<T>, T> Serialize for Varsig<V, C, T> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let mut bytes = Vec::new();

        // Varsig tag
        leb128::write::unsigned(&mut bytes, 0x34).map_err(|e| {
            serde::ser::Error::custom(format!(
                "unable to varsig prefix tag write into new owned vec: {e}"
            ))
        })?;

        // Version tag
        leb128::write::unsigned(&mut bytes, 0x01).map_err(|e| {
            serde::ser::Error::custom(format!(
                "unable to write varsig version tag into owned vec with one element: {e}"
            ))
        })?;

        // Signature tag
        leb128::write::unsigned(&mut bytes, self.verifier_cfg.prefix()).map_err(|e| {
            serde::ser::Error::custom(format!(
                "unable to write verifier prefix tag into owned vec: {e}"
            ))
        })?;

        for segment in &self.verifier_cfg.config_tags() {
            leb128::write::unsigned(&mut bytes, *segment).map_err(|e| {
                serde::ser::Error::custom(format!(
                    "unable to write varsig config segment into owned vec {segment}: {e}",
                ))
            })?;
        }

        // Codec tag
        leb128::write::unsigned(&mut bytes, self.codec.multicodec_code()).map_err(|e| {
            serde::ser::Error::custom(format!(
                "unable to write varsig version tag into owned vec with one element: {e}"
            ))
        })?;

        serializer.serialize_bytes(&bytes)
    }
}

impl<'de, V: Verify, C: Codec<T>, T> Deserialize<'de> for Varsig<V, C, T> {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let bytes: serde_bytes::ByteBuf =
            serde::Deserialize::deserialize(deserializer).map_err(|e| {
                serde::de::Error::custom(format!("unable to deserialize varsig header: {e}"))
            })?;

        let mut cursor = std::io::Cursor::new(bytes.as_slice());
        let len = bytes.len() as u64;

        let varsig_tag = leb128::read::unsigned(&mut cursor).map_err(|e| {
            serde::de::Error::custom(format!("unable to read leb128 unsigned: {e}"))
        })?;
        if varsig_tag != 0x34 {
            return Err(serde::de::Error::custom(format!(
                "expected varsig tag 0x34, found {varsig_tag:#x}"
            )));
        }

        let version_tag = leb128::read::unsigned(&mut cursor).map_err(|e| {
            serde::de::Error::custom(format!("unable to read leb128 unsigned: {e}"))
        })?;
        if version_tag != 0x01 {
            return Err(serde::de::Error::custom(format!(
                "expected varsig version tag 0x01, found {version_tag:#x}"
            )));
        }

        let mut remaining = Vec::new();
        while cursor.position() < len {
            let seg = leb128::read::unsigned(&mut cursor).map_err(|e| {
                serde::de::Error::custom(format!("unable to read leb128 unsigned segment: {e}"))
            })?;
            remaining.push(seg);
        }

        let (verifier_cfg, more) = V::try_from_tags(remaining.as_slice())
            .ok_or_else(|| serde::de::Error::custom("unable to create verifier from tags"))?;
        let codec = C::try_from_tags(more)
            .ok_or_else(|| serde::de::Error::custom("unable to create codec from tags"))?;

        Ok(Varsig {
            verifier_cfg,
            codec,
            _data: std::marker::PhantomData,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::signature::eddsa::{Ed25519, EdDsa};

    use serde_ipld_dagcbor::codec::DagCborCodec;
    use testresult::TestResult;

    #[test]
    fn test_ed25519_varsig_header_round_trip() -> TestResult {
        let fixture = Varsig::new(EdDsa::new(), DagCborCodec);
        let dag_cbor = serde_ipld_dagcbor::to_vec(&fixture)?;
        let round_tripped: Varsig<Ed25519, DagCborCodec, String> =
            serde_ipld_dagcbor::from_slice(&dag_cbor)?;
        assert_eq!(fixture, round_tripped);
        Ok(())
    }

    #[test]
    fn test_ed25519_varsig_header_fixture() -> TestResult {
        let dag_cbor = [0x48, 0x34, 0x01, 0xed, 0x01, 0xed, 0x01, 0x13, 0x71];
        let varsig: Varsig<Ed25519, DagCborCodec, String> =
            serde_ipld_dagcbor::from_slice(&dag_cbor)?;
        assert_eq!(varsig, Varsig::new(EdDsa::new(), DagCborCodec));
        Ok(())
    }

    #[test]
    fn test_verifier_reader() -> TestResult {
        let varsig: Varsig<Ed25519, DagCborCodec, String> = Varsig::new(EdDsa::new(), DagCborCodec);
        assert_eq!(varsig.verifier_cfg(), &EdDsa::new());
        Ok(())
    }

    #[test]
    fn test_codec_reader() -> TestResult {
        let varsig: Varsig<Ed25519, DagCborCodec, String> = Varsig::new(EdDsa::new(), DagCborCodec);
        assert_eq!(varsig.codec(), &DagCborCodec);
        Ok(())
    }

    #[test]
    fn test_try_verify() -> TestResult {
        #[derive(Debug, PartialEq, Serialize, Deserialize)]
        struct TestPayload {
            message: String,
            count: u8,
        }

        let payload = TestPayload {
            message: "Hello, Varsig!".to_string(),
            count: 42,
        };

        let mut csprng = rand::thread_rng();
        let sk = ed25519_dalek::SigningKey::generate(&mut csprng);
        let varsig: Varsig<Ed25519, DagCborCodec, TestPayload> =
            Varsig::new(EdDsa::new(), DagCborCodec);

        let (sig, _encoded) = varsig.try_sign(&sk, &payload)?;
        varsig.try_verify(&sk.verifying_key(), &payload, &sig)?;

        Ok(())
    }
}
