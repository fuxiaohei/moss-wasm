#!/usr/bin/env bash

echo -e "rust-basic:wasm32:"
cargo build -p rust-basic --target wasm32-unknown-unknown --release && cargo run --release rust-basic --wasi=false

echo -e "rust-fetch:wasm32:"
cargo build -p rust-fetch --target wasm32-unknown-unknown --release && cargo run --release rust-fetch --wasi=false

echo -e "rust-basic:wasi:"
cargo build -p rust-basic --target wasm32-wasi --release && cargo run --release rust-basic

echo -e "\nrust-fetch:wasi:"
cargo build -p rust-fetch --target wasm32-wasi --release && cargo run --release rust-fetch