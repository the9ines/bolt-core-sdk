#!/usr/bin/env bash
set -euo pipefail

# Cross-language BTR constants verification for bolt-core-sdk.
# Extracts 5 HKDF info strings, BTR_KEY_LENGTH, and 4 BTR wire error codes
# from both Rust (bolt-btr/src/constants.rs) and TypeScript (btr/constants.ts)
# and asserts they match.
#
# Rust is canonical. TS MUST match.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

RUST_SRC="$ROOT/rust/bolt-btr/src/constants.rs"
TS_SRC="$ROOT/ts/bolt-core/src/btr/constants.ts"

fail=0

echo "=== BTR Constants Verification ==="

# --- HKDF info strings ---
hkdf_names=(
  BTR_SESSION_ROOT_INFO
  BTR_TRANSFER_ROOT_INFO
  BTR_MESSAGE_KEY_INFO
  BTR_CHAIN_ADVANCE_INFO
  BTR_DH_RATCHET_INFO
)

for name in "${hkdf_names[@]}"; do
  # Rust: pub const NAME: &[u8] = b"value";  (only match the pub const line)
  rust_val=$(grep "^pub const ${name}" "$RUST_SRC" | sed -n 's/.*b"\([^"]*\)".*/\1/p')
  # TS: export const NAME = 'value';
  ts_val=$(grep "^export const ${name}" "$TS_SRC" | sed -n "s/.*= '\([^']*\)'.*/\1/p")

  echo "  ${name}: Rust=${rust_val}  TS=${ts_val}"

  if [ "$rust_val" != "$ts_val" ]; then
    echo "FAIL: ${name} mismatch (Rust=${rust_val}, TS=${ts_val})"
    fail=1
  fi
done

# --- BTR_KEY_LENGTH ---
rust_kl=$(grep '^pub const BTR_KEY_LENGTH' "$RUST_SRC" | grep -oE '[0-9]+' | head -1)
ts_kl=$(grep '^export const BTR_KEY_LENGTH' "$TS_SRC" | grep -oE '[0-9]+' | head -1)
echo "  BTR_KEY_LENGTH: Rust=${rust_kl}  TS=${ts_kl}"

if [ "$rust_kl" != "$ts_kl" ]; then
  echo "FAIL: BTR_KEY_LENGTH mismatch (Rust=${rust_kl}, TS=${ts_kl})"
  fail=1
fi

# --- BTR wire error codes ---
# Extract RATCHET_* codes from the array declarations only.
# Rust: pub const BTR_WIRE_ERROR_CODES: [&str; 4] = [ "CODE", ... ];
rust_codes=$(sed -n '/BTR_WIRE_ERROR_CODES/,/];/p' "$RUST_SRC" | grep -oE '"RATCHET_[A-Z_]+"' | tr -d '"' | sort)
# TS: export const BTR_WIRE_ERROR_CODES = [ 'CODE', ... ] as const;
ts_codes=$(sed -n '/BTR_WIRE_ERROR_CODES/,/] as const/p' "$TS_SRC" | grep -oE "'RATCHET_[A-Z_]+'" | tr -d "'" | sort)

echo "  BTR wire error codes (Rust): $(echo $rust_codes | tr '\n' ' ')"
echo "  BTR wire error codes (TS):   $(echo $ts_codes | tr '\n' ' ')"

if [ "$rust_codes" != "$ts_codes" ]; then
  echo "FAIL: BTR wire error codes mismatch"
  fail=1
fi

# --- Count check: 4 BTR error codes ---
rust_count=$(echo "$rust_codes" | wc -w | tr -d ' ')
ts_count=$(echo "$ts_codes" | wc -w | tr -d ' ')

if [ "$rust_count" != "4" ]; then
  echo "FAIL: Expected 4 Rust BTR error codes, got ${rust_count}"
  fail=1
fi

if [ "$ts_count" != "4" ]; then
  echo "FAIL: Expected 4 TS BTR error codes, got ${ts_count}"
  fail=1
fi

if [ "$fail" -eq 0 ]; then
  echo "PASS: all BTR constants aligned (5 HKDF info + key length + 4 error codes)"
else
  exit 1
fi
