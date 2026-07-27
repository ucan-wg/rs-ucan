# DID

Decentralized Identifiers are the principal type in UCAN, identifying issuers and audiences without relying on a central authority.

## Overview

The `Did` trait abstracts over DID methods so that the rest of the crate operates generically over any identifier scheme. A concrete implementation, `Ed25519Did`, encodes an Ed25519 verifying key as a `did:key` string.

The `DidSigner` trait pairs a `Did` with its signing key, providing a single value that can both identify and act on behalf of a principal.

## The `Did` Trait

```rust
trait Did:
    PartialEq + ToString + FromStr + Serialize + Deserialize + Debug
{
    type VarsigConfig: Sign + Clone;

    fn did_method(&self) -> &str;
    fn varsig_config(&self) -> &Self::VarsigConfig;
}
```

| Bound | Purpose |
|-------|---------|
| `PartialEq` | Compare two DIDs for equality |
| `ToString` | Render the canonical string form (e.g. `did:key:z...`) |
| `FromStr` | Parse a DID from its string form |
| `Serialize` / `Deserialize` | Round-trip through serde (DAG-CBOR, DAG-JSON) |
| `Debug` | Diagnostic output |

The associated type `VarsigConfig` links a DID to the signature algorithm it uses. This is how the envelope layer knows _which_ Varsig configuration to use when signing or verifying.

## The `DidSigner` Trait

```rust
trait DidSigner {
    type Did: Did + Clone;

    fn did(&self) -> &Self::Did;
    fn signer(&self) -> &<<Self::Did as Did>::VarsigConfig as Sign>::Signer;
}
```

`DidSigner` is deliberately separate from `Did`. A DID is a _public_ identifier that anyone can hold. A signer is a _secret_ capability that only the key holder possesses. Keeping them in distinct traits prevents accidental exposure of signing keys in contexts that only need identification.

## `Ed25519Did`

The concrete `did:key` implementation for Ed25519.

```rust
struct Ed25519Did(ed25519_dalek::VerifyingKey, Ed25519);
```

It wraps an `ed25519_dalek::VerifyingKey` and an `Ed25519` Varsig configuration. The struct derives `Clone`, `Copy`, and `PartialEq`.

### Wire Format

A `did:key` string is built from three layers:

```
did:key:z6Mk...
│   │   │└──── base58btc-encoded payload
│   │   └───── multibase prefix 'z' (base58btc)
│   └───────── DID method
└───────────── DID scheme
```

The base58btc payload contains a multicodec prefix followed by the raw key bytes:

```
┌──────┬──────┬────────────────────────────────┐
│ 0xED │ 0x01 │  32 bytes: Ed25519 public key  │
│      │      │                                │
└──────┴──────┴────────────────────────────────┘
  multicodec         raw verifying key
  ed25519-pub        (compressed Edwards point)
  prefix
```

Total payload: 34 bytes (2-byte header + 32-byte key).

### Encoding (`Display`)

1. Allocate a 34-byte buffer.
2. Push `0xED`, `0x01`.
3. Append the 32-byte verifying key.
4. Base58btc-encode the buffer.
5. Format as `did:key:z{base58}`.

### Decoding (`FromStr`)

1. Split on `:` and verify the `did:key` prefix.
2. Strip the `z` multibase prefix.
3. Base58btc-decode to 34 bytes.
4. Verify the multicodec header is `[0xED, 0x01]`.
5. Construct a `VerifyingKey` from the remaining 32 bytes.

### Conversions

`Ed25519Did` implements `From` for both key types:

| From | Behaviour |
|------|-----------|
| `ed25519_dalek::VerifyingKey` | Wraps directly |
| `ed25519_dalek::SigningKey` | Extracts the verifying key, then wraps |

## Serde

`Ed25519Did` serializes as its `did:key:z...` string form.

Deserialization uses a custom `Visitor` rather than `String::deserialize` + `FromStr`. The `DidKeyVisitor` validates the structure incrementally:

```rust
// Inside Deserialize impl
struct DidKeyVisitor;

impl Visitor<'_> for DidKeyVisitor {
    type Value = Ed25519Did;

    fn visit_str<E: de::Error>(self, v: &str) -> Result<Ed25519Did, E> {
        // 1. Check "did:key:z" prefix
        // 2. Base58btc-decode the remainder
        // 3. Verify length == 34
        // 4. Verify multicodec header [0xED, 0x01]
        // 5. Construct VerifyingKey from bytes 2..34
    }
}
```

> [!NOTE]
> Using a `Visitor` avoids allocating an intermediate `String` when the deserializer can provide a borrowed `&str`. This matters in `no_std` contexts where allocations are expensive.

## `Ed25519Signer`

```rust
struct Ed25519Signer {
    did: Ed25519Did,
    signer: ed25519_dalek::SigningKey,
}
```

Constructed from a `SigningKey`, it derives the `Ed25519Did` automatically. Serialization delegates to the inner `Ed25519Did` — the signing key is _never_ included in the wire format.

| Method | Returns |
|--------|---------|
| `Ed25519Signer::new(signing_key)` | Derives the verifying key and wraps both |
| `.did()` | `&Ed25519Did` |
| `.signer()` | `&ed25519_dalek::SigningKey` |

`Ed25519Signer` also implements `From<ed25519_dalek::SigningKey>` and `Display` (delegates to the DID string).

## Error Types

Parsing an `Ed25519Did` from a string can fail in four ways:

| Variant | Meaning |
|---------|---------|
| `InvalidDidHeader` | Missing or malformed `did:key:` prefix |
| `MissingBase58Prefix` | The multibase prefix `z` is absent |
| `InvalidBase58` | Base58btc decoding failed |
| `InvalidKey` | Wrong byte length, wrong multicodec header, or invalid curve point |

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Error)]
enum Ed25519DidFromStrError {
    #[error("invalid did header")]
    InvalidDidHeader,

    #[error("missing base58 prefix 'z'")]
    MissingBase58Prefix,

    #[error("invalid base58 encoding")]
    InvalidBase58,

    #[error("invalid key bytes")]
    InvalidKey,
}
```
