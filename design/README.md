# rs-ucan Design

This directory describes the design and architecture of the rs-ucan implementation of [UCAN v1.0.0-rc.1](https://github.com/ucan-wg/spec).

| Document                         | Purpose                                                 |
|----------------------------------|---------------------------------------------------------|
| [varsig.md](./varsig.md)         | Varsig header, codec layer, and signature abstraction   |
| [envelope.md](./envelope.md)     | Signed envelope format and DAG-CBOR serialization       |
| [delegation.md](./delegation.md) | Delegation payload, subject types, and chain semantics  |
| [policy.md](./policy.md)         | Policy predicate engine and jq-inspired selectors       |
| [invocation.md](./invocation.md) | Invocation payload, promise types, and chain validation |
| [did.md](./did.md)               | DID abstraction and `did:key` (Ed25519) implementation  |
| [no_std.md](./no_std.md)         | `no_std` strategy, feature gates, and platform support  |

## Architecture

```mermaid
block-beta
  columns 1
  block:app["Application"]
    Invocation
    Delegation
  end
  block:auth["Authorization"]
    Policy
    DelegationStore
  end
  block:envelope["Envelope"]
    Envelope
    PayloadTag
  end
  block:crypto["Crypto"]
    Varsig
    DID
    CID
    Nonce
  end
  block:codec["Codec"]
    DagCBOR["DAG-CBOR"]
    DagJSON["DAG-JSON (std)"]
  end

  app --> auth
  auth --> envelope
  envelope --> crypto
  crypto --> codec
```

## Crate Structure

```
rs-ucan/
  varsig/       Signature metadata layer (no_std)
  ucan/         Core UCAN implementation (no_std)
  ucan_wasm/    Wasm bindings (stub)
```

The dependency direction is:

```
ucan_wasm → ucan → varsig
```

## Data Flow

```mermaid
sequenceDiagram
    participant Builder as DelegationBuilder
    participant Payload as DelegationPayload
    participant Codec as DAG-CBOR
    participant Varsig as Varsig<Ed25519>
    participant Envelope as Envelope

    Builder->>Payload: .into_payload()
    Payload->>Codec: encode(payload)
    Codec->>Varsig: sign(encoded_bytes, signing_key)
    Varsig-->>Envelope: (signature, {h: header, tag: payload})

    Note over Envelope: Serialized as CBOR 2-tuple
```

## Delegation Chain

```mermaid
sequenceDiagram
    participant Root as Root Authority
    participant Del1 as Delegatee 1
    participant Del2 as Delegatee 2
    participant Inv as Invoker

    Root->>Del1: Delegation (sub: Root, cmd: /*)
    Del1->>Del2: Delegation (sub: Root, cmd: /crud/*)
    Del2->>Inv: Delegation (sub: Root, cmd: /crud/read, pol: [.path == "/public"])
    Inv->>Inv: Invocation (sub: Root, cmd: /crud/read, arg: {path: "/public"})

    Note over Inv: syntatic_checks() walks the chain:<br/>1. Principal alignment (iss/aud)<br/>2. Command hierarchy (starts_with)<br/>3. Policy predicates (run against args)
```

## Spec Version

This implementation targets:

| Spec                                                     | Version     |
|----------------------------------------------------------|-------------|
| [UCAN](https://github.com/ucan-wg/spec)                  | v1.0.0-rc.1 |
| [UCAN Delegation](https://github.com/ucan-wg/delegation) | v1.0.0-rc.1 |
| [Varsig](https://github.com/ChainAgnostic/varsig)        | Draft       |

## Design Principles

- _`no_std` first_ — both crates compile without `std` (`alloc` only)
- _Type-driven_ — builders enforce required fields at compile time via phantom types
- _Parse, don't validate_ — `Command::parse()`, `Timestamp::from_unix()`, `DID::from_str()` return structured types that make invalid states unrepresentable
- _Codec agnostic_ — the `Codec<T>` trait abstracts over DAG-CBOR/DAG-JSON; signature verification works against any codec
- _Algorithm agnostic_ — the `Verify`/`Sign` traits abstract over Ed25519, ECDSA (P-256/P-384/P-521), and WebCrypto composites
- _Content addressed_ — delegations and invocations are identified by their CID (CIDv1, SHA-256, DAG-CBOR)
