#!/bin/zsh
cargo build --package debugger --target wasm32-unknown-unknown
cargo build --package plugin_test --target wasm32-unknown-unknown
cargo run --package opvm2_cli -- "$@"
