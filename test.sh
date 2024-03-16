#!/bin/zsh

cargo build --package debugger --target wasm32-unknown-unknown
cargo build --package plugin_test --target wasm32-unknown-unknown
#cargo build --package opvm2
#cargo build --package opvm2_vm

cargo test --workspace -q
