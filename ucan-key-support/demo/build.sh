#!/bin/bash

cargo build --release --target="wasm32-unknown-unknown" --features=web

wasm-bindgen \
    --target web \
    --out-dir static \
    ../../target/wasm32-unknown-unknown/release/ucan_key_support.wasm
