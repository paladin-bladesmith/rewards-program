#!/usr/bin/env bash

set -e
cd "$(dirname "$0")/.."

source ./ci/rust-version.sh stable
source ./ci/solana-version.sh

export RUSTFLAGS="-D warnings"
export RUSTBACKTRACE=1

set -x

cargo +"$rust_stable" build
cargo +"$rust_stable" test -- --nocapture
cargo +"$rust_stable" test-sbf --features bpf-entrypoint -- --nocapture

exit 0
