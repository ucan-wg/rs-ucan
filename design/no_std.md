# `no_std` Support

Both `varsig` and `ucan` compile as `no_std` + `alloc` by default, with `std` as a default-on feature gate.

## Overview

The crate root of each workspace member declares:

```rust
#![cfg_attr(not(feature = "std"), no_std)]
extern crate alloc;
```

The `std` feature is listed in `default` for both crates. Downstream consumers that need `no_std` opt out with `default-features = false` and enable only the features they need.

The guiding principle is: _everything that can work without `std` does work without `std`_. Only APIs that inherently require OS services (`SystemTime`, `Mutex`, `HashMap`) are gated.

## Feature Gates

### `ucan`

| Feature | Default | Enables |
|---------|---------|---------|
| `std` | yes | `Timestamp::now()`, `Arc<Mutex<HashMap>>` store, `HashMap`/`HashSet` collection impls, propagates `std` to all deps |
| `getrandom` | implied by `std` | `Nonce::generate_16()` via CSPRNG |
| `test_utils` | no | `arb` + `property_test` |
| `arb` | no | `Arbitrary` impls |
| `property_test` | no | `proptest` + `proptest-arbitrary-interop` |

### `varsig`

| Feature | Default | Enables |
|---------|---------|---------|
| `std` | yes | Upstream `codec` module in `ipld-core`, `serde_ipld_dagcbor::codec::DagCborCodec` re-export, propagates `std` to deps |
| `dag_cbor` | yes | DAG-CBOR codec via `serde_ipld_dagcbor` |
| `dag_json` | no | DAG-JSON codec via `serde_ipld_dagjson` — _implies `std`_ |
| `dag_pb` | no | DAG-PB codec via `ipld-dagpb` + `bytes` |
| `ed25519` | yes | Ed25519 signature support via `ed25519-dalek` |
| `web_crypto` | no | ES256 + ES384 + ES512 + Ed25519 composite enum |

> [!NOTE]
> `dag_json` forces `std` because the upstream `serde_ipld_dagjson` crate has no `no_std` support.

## What's Available in `no_std`

The full UCAN workflow operates without `std`:

| Capability | Module |
|------------|--------|
| Delegation creation and serialization | `delegation` |
| Invocation creation and serialization | `invocation` |
| Delegation chain validation | `invocation::syntatic_checks` |
| Signing and verification | `DelegationBuilder::try_build`, `InvocationBuilder::try_build` |
| `DelegationStore` trait | `delegation::store` |
| `Rc<RefCell<BTreeMap>>` store impl | `delegation::store` |
| Policy predicates and selectors | `delegation::policy` |
| DID types (`Ed25519Did`) | `did` |
| CID computation (CIDv1, SHA-256, DAG-CBOR) | `cid` |
| Async abstractions (`Send` / `!Send`) | `future_form`, `futures` |
| Nonce from raw bytes | `Nonce::from_bytes(&[u8])` |
| DAG-CBOR encoding and decoding | `varsig::codec::DagCborCodec` |
| Varsig header parsing and serialization | `varsig::header` |

## What Requires `std`

| API | Why |
|-----|-----|
| `Timestamp::now()` | Calls `std::time::SystemTime::now()` |
| `Timestamp::five_minutes_from_now()` | Same |
| `Timestamp::five_years_from_now()` | Same |
| `TryFrom<SystemTime> for Timestamp` | `SystemTime` is `std`-only |
| `Arc<Mutex<HashMap>>` store | `Mutex` is `std`-only |
| `Rc<RefCell<HashMap>>` store | `HashMap` requires `std` (or `hashbrown`) |
| `HashMap`/`HashSet` collection aliases | Sourced from `std::collections` |
| DAG-JSON codec | Upstream `serde_ipld_dagjson` requires `std` |
| `Nonce::generate_16()` | Requires `getrandom` (implied by `std`) |

## Collections Abstraction

`ucan/src/collections.rs` provides conditional type aliases:

```rust
// std
pub use std::collections::{hash_map::Entry, HashMap as Map, HashSet as Set};

// no_std
pub use alloc::collections::{btree_map::Entry, BTreeMap as Map, BTreeSet as Set};
```

| Mode | `Map` | `Set` | `Entry` | Lookup |
|------|-------|-------|---------|--------|
| `std` | `HashMap` | `HashSet` | `hash_map::Entry` | O(1) amortized |
| `no_std` | `BTreeMap` | `BTreeSet` | `btree_map::Entry` | O(log n) |

This trades constant-time lookup for ordered-map lookup in `no_std`, avoiding a `hashbrown` dependency. Types where ordering is semantically required (e.g., `BTreeMap<String, Ipld>` in delegation payloads) use `BTreeMap` directly regardless of features.

## Dependency Strategy

Every dependency is configured with `default-features = false` and only the features needed for `no_std` + `alloc`. The `std` feature propagates `std` to each dep.

### `ucan` Dependencies

| Crate | Base Config | `std` Adds |
|-------|-------------|------------|
| `bs58` | `default-features = false, features = ["alloc"]` | `bs58/std` |
| `ed25519-dalek` | (default) | `ed25519-dalek/std` |
| `ipld-core` | `default-features = false, features = ["serde"]` | `ipld-core/std` |
| `nom` | `default-features = false, features = ["alloc"]` | `nom/std` |
| `serde` | `default-features = false, features = ["derive", "alloc"]` | `serde/std` |
| `serde_bytes` | `default-features = false, features = ["alloc"]` | `serde_bytes/std` |
| `serde_ipld_dagcbor` | `default-features = false` | `serde_ipld_dagcbor/std` |
| `sha2` | `default-features = false` | `sha2/std` |
| `signature` | `default-features = false` | `signature/std` |
| `thiserror` | `default-features = false` | `thiserror/std` |
| `tracing` | `default-features = false, features = ["attributes"]` | `tracing/std` |
| `varsig` | `default-features = false, features = ["dag_cbor", "ed25519"]` | `varsig/std` |

### `varsig` Dependencies

| Crate | Base Config | `std` Adds |
|-------|-------------|------------|
| `async-signature` | `default-features = false` | — |
| `ipld-core` | `default-features = false, features = ["serde"]` | `ipld-core/std`, `ipld-core/codec` |
| `serde` | `default-features = false, features = ["derive", "alloc"]` | `serde/std` |
| `serde_bytes` | `default-features = false, features = ["alloc"]` | — |
| `serde_ipld_dagcbor` | `default-features = false` (optional) | `serde_ipld_dagcbor/std`, `serde_ipld_dagcbor/codec` |
| `signature` | `default-features = false` | — |
| `thiserror` | `default-features = false` | `thiserror/std` |
| `tracing` | `default-features = false, features = ["attributes"]` | `tracing/std` |

## `DagCborCodec` Dual Definition

The `DagCborCodec` type has two definitions depending on the `std` feature:

| Mode | Definition | Error Types |
|------|-----------|-------------|
| `std` | Re-exported from `serde_ipld_dagcbor::codec::DagCborCodec` | `serde_ipld_dagcbor::error::CodecError` |
| `no_std` | Local unit struct in `varsig::codec` | `DagCborEncodeError` / `DagCborDecodeError` newtypes |

The multicodec code `0x71` is hardcoded rather than sourced from `ipld_core::codec::Codec::CODE` because the `Codec` trait in `ipld-core` is gated behind `#[cfg(all(feature = "std", feature = "codec"))]`.

> [!NOTE]
> `0x71` (DAG-CBOR) and `0x0129` (DAG-JSON) are stable IANA multicodec values that will never change.

## Crate Replacements

Several crates were replaced to eliminate `std` dependencies:

| Original | Replacement | Reason |
|----------|-------------|--------|
| `base58` 0.2 | `bs58` 0.5 | `base58` requires `std`; `bs58` supports `no_std` + `alloc` |
| `leb128` 0.2.5 | `leb128fmt` 0.1.0 | `leb128` uses `std::io`; `leb128fmt` is `no_std`, zero-dep, array/slice API |
| `serde-value` 0.7 | Custom serde visitor | `serde-value` requires `std`; ~20 lines of visitor code |
| `thiserror` 1.x | `thiserror` 2.x | 2.x supports `core::error::Error` (Rust 1.81+) for `no_std` |

## Upstream Blockers

Several upstream crates prevent a fully clean `no_std` build on real embedded targets.

| Crate | Issue | Impact | Workaround |
|-------|-------|--------|------------|
| `ipld-core` | `codec` module gated behind `std` | Cannot use `ipld_core::codec::Codec::CODE` constant | Hardcode multicodec values `0x71`, `0x0129` |
| `ipld-core` | Pulls in `serde_bytes` with `std` feature in some configurations | Transitive `std` dependency | Pin `serde_bytes` with `default-features = false, features = ["alloc"]` |
| `serde_ipld_dagcbor` | Error types only impl `std::error::Error` | Cannot use upstream error types in `no_std` | Newtype wrappers `DagCborEncodeError` / `DagCborDecodeError` with `core::error::Error` impls |
| `serde_ipld_dagjson` | No `no_std` support at all | DAG-JSON unavailable in `no_std` | `dag_json` feature implies `std` |
| `leb128fmt` | Error type uses `std::error::Error` | — | Use `core::error::Error` (stabilized in Rust 1.81, MSRV is 1.90) |

> [!WARNING]
> These blockers mean that `cargo build --no-default-features` _succeeds on the host triple_ but does not guarantee a successful build on an actual embedded target (e.g., `thumbv7em-none-eabihf`). Transitive dependencies may pull in `std` symbols that only fail to link on targets without `std`.

## Platform Support

| Target | Status | Notes |
|--------|--------|-------|
| `x86_64-unknown-linux-gnu` | works | Default, `std` enabled |
| `x86_64-unknown-linux-gnu` `--no-default-features` | works | `no_std` + `alloc` on host |
| `wasm32-unknown-unknown` | blocked | Timestamp Wasm block references undefined symbols; `ucan_wasm` does not compile |
| `thumbv7em-none-eabihf` | blocked | Upstream transitive `std` deps fail to link |
| Any target with `getrandom` support | `Nonce::generate_16()` available | Enable `getrandom` feature independently |

The path to real embedded support:

1. Upstream `ipld-core` exposes `Codec::CODE` without `std`
2. Upstream `serde_ipld_dagcbor` implements `core::error::Error` on its error types
3. Fix the `wasm32` timestamp block (references `wasm_bindgen` without imports)
4. Verify the full transitive closure links on a `no_std` target

## `getrandom` as Independent Feature

`getrandom` and `rand` are both `#![no_std]` crates. They work on Wasm, ARM Cortex, x86 with RDRAND, and other platforms with entropy sources. The `getrandom` feature is implied by `std` but can be enabled independently:

```toml
[dependencies]
ucan = { version = "0.5", default-features = false, features = ["getrandom"] }
```

This gives `Nonce::generate_16()` on `no_std` platforms that have a CSPRNG. Platforms without `getrandom` can still construct nonces via `Nonce::from_bytes(&[u8])`.
