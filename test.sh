#!/bin/zsh

cargo build --package debugger --target wasm32-unknown-unknown
cargo build --package plugin_test --target wasm32-unknown-unknown
cargo build --package opvm2
cargo build --package opvm2_vm

cargo test --package debugger
cargo test --package opvm2
cargo test --package opvm2_vm
