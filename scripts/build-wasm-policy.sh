#!/usr/bin/env bash
# T-STREAM-1: Build WASM policy bindings for bolt-transport-web.
#
# Uses rustup's toolchain explicitly to avoid Homebrew rustc conflicts.
# Output: ts/bolt-transport-web/wasm/
#
# Usage:
#   ./scripts/build-wasm-policy.sh          # build only
#   ./scripts/build-wasm-policy.sh --gate   # build + size gate check

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SDK_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
WASM_CRATE="$SDK_ROOT/rust/bolt-transfer-policy-wasm"
OUT_DIR="$SDK_ROOT/ts/bolt-transport-web/wasm"

# Size gate: combined gzipped <= 80 KB
SIZE_BUDGET_BYTES=81920

echo "[WASM] Building bolt-transfer-policy-wasm..."

# Ensure wasm-pack is available
if ! command -v wasm-pack &>/dev/null && ! command -v "$HOME/.cargo/bin/wasm-pack" &>/dev/null; then
    echo "[WASM] ERROR: wasm-pack not found. Install: cargo install wasm-pack"
    exit 1
fi
WASM_PACK="${HOME}/.cargo/bin/wasm-pack"
if ! command -v "$WASM_PACK" &>/dev/null; then
    WASM_PACK="wasm-pack"
fi

# Build with rustup toolchain
cd "$WASM_CRATE"
PATH="$HOME/.rustup/toolchains/stable-aarch64-apple-darwin/bin:$PATH" \
    "$WASM_PACK" build --target web --release --out-dir "$OUT_DIR" 2>&1

# Clean up wasm-pack artifacts we don't need
rm -f "$OUT_DIR/.gitignore" "$OUT_DIR/package.json"

echo ""
echo "[WASM] Build complete. Output: $OUT_DIR"

# Size report
WASM_FILE="$OUT_DIR/bolt_transfer_policy_wasm_bg.wasm"
JS_FILE="$OUT_DIR/bolt_transfer_policy_wasm.js"

WASM_RAW=$(wc -c < "$WASM_FILE" | tr -d ' ')
WASM_GZ=$(gzip -c "$WASM_FILE" | wc -c | tr -d ' ')
JS_RAW=$(wc -c < "$JS_FILE" | tr -d ' ')
JS_GZ=$(gzip -c "$JS_FILE" | wc -c | tr -d ' ')
COMBINED_GZ=$((WASM_GZ + JS_GZ))

echo ""
echo "┌─────────────────────────────────────────┐"
echo "│ T-STREAM-1 WASM Size Report             │"
echo "├──────────────────────┬──────────┬────────┤"
printf "│ %-20s │ %8s │ %6s │\n" "File" "Raw" "Gzip"
echo "├──────────────────────┼──────────┼────────┤"
printf "│ %-20s │ %6s B │ %4s B │\n" "*.wasm" "$WASM_RAW" "$WASM_GZ"
printf "│ %-20s │ %6s B │ %4s B │\n" "*.js (glue)" "$JS_RAW" "$JS_GZ"
echo "├──────────────────────┼──────────┼────────┤"
printf "│ %-20s │          │ %4s B │\n" "Combined gzip" "$COMBINED_GZ"
printf "│ %-20s │          │ %4s B │\n" "Budget" "$SIZE_BUDGET_BYTES"
echo "└──────────────────────┴──────────┴────────┘"

if [[ "${1:-}" == "--gate" ]]; then
    echo ""
    if (( COMBINED_GZ > SIZE_BUDGET_BYTES )); then
        echo "[SIZE_GATE] FAIL: ${COMBINED_GZ} > ${SIZE_BUDGET_BYTES} bytes"
        exit 1
    else
        echo "[SIZE_GATE] PASS: ${COMBINED_GZ} <= ${SIZE_BUDGET_BYTES} bytes"
    fi
fi
