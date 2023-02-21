#!/usr/bin/env bash

cmd="target/release/moss-wasm"
echo -e "build runner:"
cargo build --release
cp $cmd moss-wasm-runner

cli="target/release/moss-cli"
echo -e "build cli:"
cargo build -p moss-cli --release
cp $cli moss-cli-bin


echo -e "js-basic:"
(cd examples/js-basic && ../../moss-cli-bin build)
./moss-wasm-runner js-basic

echo -e "js-fetch:"
(cd examples/js-fetch && ../../moss-cli-bin build)
./moss-wasm-runner js-fetch

echo -e "rust-basic:wasm32:"
cargo build -p rust-basic --target wasm32-unknown-unknown --release && $cmd rust-basic --wasi=false

echo -e "rust-fetch:wasm32:"
cargo build -p rust-fetch --target wasm32-unknown-unknown --release && $cmd rust-fetch --wasi=false

echo -e "rust-basic:wasi:"
cargo build -p rust-basic --target wasm32-wasi --release && $cmd rust-basic

echo -e "\nrust-fetch:wasi:"
cargo build -p rust-fetch --target wasm32-wasi --release && $cmd rust-fetch

echo -e "\nrust-kv:wasi:"
cargo build -p rust-kv --target wasm32-wasi --release && $cmd rust-kv

rm -f moss-cli-bin moss-wasm-runner
