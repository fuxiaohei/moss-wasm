#!/usr/bin/env bash

echo -e "build runner":
cargo build --release
ls -lh target/release/moss-wasm

echo -e "rust-basic:wasm32:"
cargo build -p rust-basic --target wasm32-unknown-unknown --release && target/release/moss-wasm rust-basic --wasi=false

echo -e "rust-fetch:wasm32:"
cargo build -p rust-fetch --target wasm32-unknown-unknown --release && target/release/moss-wasm rust-fetch --wasi=false

echo -e "rust-basic:wasi:"
cargo build -p rust-basic --target wasm32-wasi --release && target/release/moss-wasm rust-basic

echo -e "\nrust-fetch:wasi:"
cargo build -p rust-fetch --target wasm32-wasi --release && target/release/moss-wasm rust-fetch

echo -e "\nrust-kv:wasi:"
cargo build -p rust-kv --target wasm32-wasi --release && target/release/moss-wasm rust-kv