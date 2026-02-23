#!/usr/bin/env bash
set -euo pipefail

# Verify no SAS computation exists in bolt-transport-web.
# The ONLY canonical SAS implementation is computeSas() in bolt-core.
# No SAS logic may exist in transport or product packages.
#
# Strict pattern: exact symbol name only. No broad regex to avoid false positives.

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

TARGET="$ROOT/ts/bolt-transport-web/src"

if [ ! -d "$TARGET" ]; then
  echo "SKIP: $TARGET does not exist"
  exit 0
fi

fail=0

# Check for the exact removed symbol
if grep -r "getVerificationCode" "$TARGET" --include="*.ts" 2>/dev/null; then
  echo "FAIL: getVerificationCode found in bolt-transport-web (must live in bolt-core only)"
  fail=1
fi

if [ "$fail" -eq 0 ]; then
  echo "PASS: no shadow SAS in bolt-transport-web"
else
  exit 1
fi
