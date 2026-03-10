#!/usr/bin/env bash
set -euo pipefail

# BTR cross-language conformance runner.
# Runs Rust BTR tests (unit + vector generation) and TS BTR tests,
# then emits a pass/fail summary.
#
# Usage: ./scripts/btr-conformance.sh
# Exit: 0 if all pass, 1 if any fail.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

pass=0
total=0

echo "=== BTR Conformance Suite ==="
echo ""

# --- Rust BTR tests (default features) ---
echo "[1/5] Rust BTR unit tests..."
total=$((total + 1))
if (cd "$ROOT/rust" && cargo test -p bolt-btr --quiet 2>&1); then
  echo "  PASS"
  pass=$((pass + 1))
else
  echo "  FAIL"
fi

# --- Rust BTR tests (vectors feature — generation + determinism) ---
echo "[2/5] Rust BTR vector golden tests..."
total=$((total + 1))
if (cd "$ROOT/rust" && cargo test -p bolt-btr --features vectors --quiet -- --test-threads=1 2>&1); then
  echo "  PASS"
  pass=$((pass + 1))
else
  echo "  FAIL"
fi

# --- TS BTR tests ---
echo "[3/5] TypeScript BTR tests..."
total=$((total + 1))
if (cd "$ROOT/ts/bolt-core" && npm run test -- --reporter=dot 2>&1); then
  echo "  PASS"
  pass=$((pass + 1))
else
  echo "  FAIL"
fi

# --- BTR constants parity ---
echo "[4/5] BTR constants cross-language parity..."
total=$((total + 1))
if "$ROOT/scripts/verify-btr-constants.sh" 2>&1; then
  echo "  PASS"
  pass=$((pass + 1))
else
  echo "  FAIL"
fi

# --- Core constants parity ---
echo "[5/5] Core constants cross-language parity..."
total=$((total + 1))
if "$ROOT/scripts/verify-constants.sh" 2>&1; then
  echo "  PASS"
  pass=$((pass + 1))
else
  echo "  FAIL"
fi

echo ""
echo "=== Summary: ${pass}/${total} passed ==="

if [ "$pass" -eq "$total" ]; then
  echo "BTR CONFORMANCE: PASS"
  exit 0
else
  echo "BTR CONFORMANCE: FAIL"
  exit 1
fi
