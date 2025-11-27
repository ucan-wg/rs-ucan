//! Top-level Varsig envelope.

use ipld_core::ipld::Ipld;
use serde::{
    de::{self, Deserializer, MapAccess, SeqAccess, Visitor},
    ser::{SerializeMap, SerializeTuple},
    Deserialize, Serialize,
};
use serde_ipld_dagcbor::codec::DagCborCodec;
use signature::SignatureEncoding;
use std::{fmt, marker::PhantomData};
use varsig::{header::Varsig, verify::Verify};

/// Top-level Varsig envelope type.
#[derive(Debug, Clone, PartialEq)]
pub struct Envelope<
    V: Verify<Signature = S>,
    T: Serialize + for<'ze> Deserialize<'ze>,
    S: SignatureEncoding,
>(
    /// Envelope signature.
    pub S,
    /// Varsig envelope
    pub EnvelopePayload<V, T>,
);

impl<V: Verify<Signature = S>, T: Serialize + for<'ze> Deserialize<'ze>, S: SignatureEncoding>
    Serialize for Envelope<V, T, S>
{
    fn serialize<Ser: serde::Serializer>(&self, serializer: Ser) -> Result<Ser::Ok, Ser::Error> {
        let mut seq = serializer.serialize_tuple(2)?;
        // Wrap signature bytes in serde_bytes::Bytes to ensure it serializes as CBOR bytes
        seq.serialize_element(&serde_bytes::Bytes::new(self.0.to_bytes().as_ref()))?;
        seq.serialize_element(&self.1)?;
        seq.end()
    }
}

impl<
        'de,
        V: Verify<Signature = S>,
        T: Serialize + for<'ze> Deserialize<'ze>,
        S: SignatureEncoding + for<'ze> Deserialize<'ze>,
    > Deserialize<'de> for Envelope<V, T, S>
{
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        struct EnvelopeVisitor<V, T, S>
        where
            V: Verify<Signature = S>,
            T: Serialize + for<'ze> Deserialize<'ze>,
            S: SignatureEncoding,
        {
            marker: std::marker::PhantomData<(V, T, S)>,
        }

        impl<'de, V, T, S> Visitor<'de> for EnvelopeVisitor<V, T, S>
        where
            V: Verify<Signature = S>,
            T: Serialize + for<'ze> Deserialize<'ze>,
            S: SignatureEncoding + Deserialize<'de>,
        {
            type Value = Envelope<V, T, S>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str("a 2-element sequence [signature, payload]")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let sig_ipld: Ipld = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(0, &self))?;

                let Ipld::Bytes(sig_bytes) = sig_ipld else {
                    return Err(de::Error::custom("expected signature to be bytes"));
                };

                let signature = S::try_from(sig_bytes.as_slice())
                    .map_err(|_| de::Error::custom("invalid signature bytes"))?;

                let payload: EnvelopePayload<V, T> = seq
                    .next_element()?
                    .ok_or_else(|| de::Error::invalid_length(1, &self))?;

                Ok(Envelope(signature, payload))
            }
        }

        deserializer.deserialize_tuple(
            2,
            EnvelopeVisitor {
                marker: std::marker::PhantomData,
            },
        )
    }
}

/// Inner Varsig envelope payload type.
#[derive(Debug, Clone, PartialEq)]
pub struct EnvelopePayload<V: Verify, T: Serialize + for<'de> Deserialize<'de>> {
    /// Varsig header.
    pub header: Varsig<V, DagCborCodec, T>,

    /// Payload data.
    pub payload: T,
}

impl<V: Verify, T: Serialize + for<'de> Deserialize<'de>> Serialize for EnvelopePayload<V, T> {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let payload = serde_value::to_value(&self.payload).map_err(serde::ser::Error::custom)?;

        let serde_value::Value::Map(payload_map) = payload else {
            return Err(serde::ser::Error::custom("payload must serialize to a map"));
        };

        // Total length = header (1) + payload (n)
        let mut map = serializer.serialize_map(Some(1 + payload_map.len()))?;
        map.serialize_entry("h", &self.header)?;

        // Flatten payload
        for (k, v) in payload_map {
            // TODO enforce that no keys conflict with "h"
            map.serialize_entry(&k, &v)?;
        }

        map.end()
    }
}

impl<'de, V, T> Deserialize<'de> for EnvelopePayload<V, T>
where
    V: Verify,
    T: Serialize + for<'any> Deserialize<'any>,
    Varsig<V, DagCborCodec, T>: Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EnvelopeVisitor<V, T>(PhantomData<(V, T)>);

        // Note the different lifetime parameter on the Visitor:
        impl<'vde, V, T> Visitor<'vde> for EnvelopeVisitor<V, T>
        where
            V: Verify,
            T: Serialize + for<'any> Deserialize<'any>,
            Varsig<V, DagCborCodec, T>: Deserialize<'vde>,
        {
            type Value = EnvelopePayload<V, T>;

            fn expecting(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                f.write_str(r#"a map with "h" and exactly one dynamic payload key"#)
            }

            fn visit_map<M>(self, mut map: M) -> Result<Self::Value, M::Error>
            where
                M: MapAccess<'vde>,
            {
                let mut header: Option<Varsig<V, DagCborCodec, T>> = None;
                let mut payload: Option<T> = None;
                let mut seen_payload = false;

                while let Some(key) = map.next_key::<&str>()? {
                    if key == "h" {
                        if header.is_some() {
                            return Err(de::Error::duplicate_field("h"));
                        }
                        let varsig_header_ipld: Ipld = map.next_value()?;
                        let varsig_header_bytes: Vec<u8> = if let Ipld::Bytes(bytes) =
                            varsig_header_ipld
                        {
                            bytes
                        } else {
                            return Err(de::Error::custom("expected varsig header to be bytes"));
                        };
                        let bytes_de = serde::de::value::BytesDeserializer::<M::Error>::new(
                            &varsig_header_bytes,
                        );

                        let varsig_header: Varsig<V, DagCborCodec, T> =
                            Varsig::<V, DagCborCodec, T>::deserialize(bytes_de)?;

                        header = Some(varsig_header);
                    } else {
                        if seen_payload {
                            return Err(de::Error::custom(
                                "expected exactly one dynamic payload entry",
                            ));
                        }
                        payload = Some(map.next_value::<T>()?);
                        seen_payload = true;
                    }
                }

                let header = header.ok_or_else(|| de::Error::missing_field("h"))?;
                let payload =
                    payload.ok_or_else(|| de::Error::custom("missing dynamic payload entry"))?;

                Ok(EnvelopePayload { header, payload })
            }
        }

        deserializer.deserialize_map(EnvelopeVisitor(PhantomData))
    }
}
