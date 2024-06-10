#!/usr/bin/env bash

set -e
cd "$(dirname "$0")/.."
source ./ci/rust-version.sh stable

cargo_audit_ignores=(
  # ed25519-dalek: Double Public Key Signing Function Oracle Attack
  #
  # Remove once SPL upgrades to Solana v1.17 or greater
  --ignore RUSTSEC-2022-0093
)
cargo +"$rust_stable" audit "${cargo_audit_ignores[@]}"
