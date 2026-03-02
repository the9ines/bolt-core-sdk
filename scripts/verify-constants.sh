#!/usr/bin/env bash
set -euo pipefail

# Cross-language constants verification for bolt-core-sdk.
# Extracts PEER_CODE_LENGTH, SAS_LENGTH, and PEER_CODE_ALPHABET from both
# the Rust and TypeScript sources and asserts they match.
#
# TS values are canonical today. Rust MUST match.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

RUST_SRC="$ROOT/rust/bolt-core/src/constants.rs"
TS_SRC="$ROOT/ts/bolt-core/src/constants.ts"

fail=0

# --- Extract Rust values ---
rust_pcl=$(grep 'PEER_CODE_LENGTH.*usize' "$RUST_SRC" | grep -oE '[0-9]+' | head -1)
rust_sal=$(grep 'SAS_LENGTH.*usize' "$RUST_SRC" | grep -oE '[0-9]+' | head -1)
rust_alpha=$(grep 'PEER_CODE_ALPHABET.*str' "$RUST_SRC" | sed -n 's/.*"\([A-Z0-9]*\)".*/\1/p')

# --- Extract TS values ---
ts_pcl=$(grep 'PEER_CODE_LENGTH' "$TS_SRC" | grep -oE '[0-9]+' | head -1)
ts_sal=$(grep 'SAS_LENGTH' "$TS_SRC" | grep -oE '[0-9]+' | head -1)
ts_alpha=$(grep 'PEER_CODE_ALPHABET' "$TS_SRC" | sed -n "s/.*'\([A-Z0-9]*\)'.*/\1/p")

# --- Compare ---
echo "=== Constants Verification ==="
echo "  PEER_CODE_LENGTH:   Rust=$rust_pcl  TS=$ts_pcl"
echo "  SAS_LENGTH:         Rust=$rust_sal  TS=$ts_sal"
echo "  PEER_CODE_ALPHABET: Rust=$rust_alpha"
echo "                      TS  =$ts_alpha"

if [ "$rust_pcl" != "$ts_pcl" ]; then
  echo "FAIL: PEER_CODE_LENGTH mismatch (Rust=$rust_pcl, TS=$ts_pcl)"
  fail=1
fi

if [ "$rust_sal" != "$ts_sal" ]; then
  echo "FAIL: SAS_LENGTH mismatch (Rust=$rust_sal, TS=$ts_sal)"
  fail=1
fi

if [ "$rust_alpha" != "$ts_alpha" ]; then
  echo "FAIL: PEER_CODE_ALPHABET mismatch"
  fail=1
fi

if [ "$fail" -eq 0 ]; then
  echo "PASS: all constants aligned"
else
  exit 1
fi
