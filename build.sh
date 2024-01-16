#!/usr/bin/bash

cargo build --features "csr" --target wasm32-unknown-unknown
cargo build --features "ssr"