#!/usr/bin/env bash
# Run scopelintup tests in an isolated temp directory.
# Usage: ./scopelintup/test-scopelintup.sh

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
SCOPELINTUP="$SCRIPT_DIR/scopelintup"

# Use a fresh temp dir each run so tests don't depend on prior state
export SCOPELINT_DIR="$(mktemp -d)"
export PATH="$SCOPELINT_DIR/bin:$PATH"

PASS=0
FAIL=0

run() {
  if "$@"; then
    echo "  OK $*"
    ((PASS++)) || true
    return 0
  else
    echo "  FAIL $*"
    ((FAIL++)) || true
    return 1
  fi
}

run_expect_fail() {
  if ! "$@" 2>/dev/null; then
    echo "  OK (expected fail) $*"
    ((PASS++)) || true
    return 0
  else
    echo "  FAIL (expected to fail) $*"
    ((FAIL++)) || true
    return 1
  fi
}

assert_output() {
  local expected="$1"
  shift
  local out
  out=$("$@")
  if [[ "$out" == *"$expected"* ]]; then
    echo "  OK $* (output contains '$expected')"
    ((PASS++)) || true
    return 0
  else
    echo "  FAIL $* (expected output containing '$expected', got: $out)"
    ((FAIL++)) || true
    return 1
  fi
}

echo "scopelintup test suite (SCOPELINT_DIR=$SCOPELINT_DIR)"
echo ""

# --- CLI ---
echo "== CLI"
run "$SCOPELINTUP" --help
assert_output "version manager" "$SCOPELINTUP" --help
run "$SCOPELINTUP" --list
assert_output "no versions installed" "$SCOPELINTUP" --list
assert_output "available releases" "$SCOPELINTUP" --list-remote
run_expect_fail "$SCOPELINTUP" --invalid 2>/dev/null
echo ""

# --- Install and shim ---
echo "== Install and shim"
run "$SCOPELINTUP" --version v0.0.21
assert_output "scopelint 0.0.21" scopelint --version
assert_output "v0.0.21" "$SCOPELINTUP" --list
run "$SCOPELINTUP" --version v0.0.20
assert_output "scopelint 0.0.20" scopelint --version
echo ""

# --- Version resolution ---
echo "== Version resolution"
PIN_DIR=$(mktemp -d)
echo "v0.0.21" > "$PIN_DIR/.scopelint-version"
out=$(cd "$PIN_DIR" && scopelint --version)
if [[ "$out" == *"0.0.21"* ]]; then
  echo "  OK .scopelint-version in project selects v0.0.21"
  ((PASS++)) || true
else
  echo "  FAIL .scopelint-version: got $out"
  ((FAIL++)) || true
fi
rm -rf "$PIN_DIR"

run "$SCOPELINTUP" --use v0.0.21
assert_output "scopelint 0.0.21" scopelint --version

PIN_DIR=$(mktemp -d)
(cd "$PIN_DIR" && run "$SCOPELINTUP" --pin v0.0.21)
[[ "$(cat "$PIN_DIR/.scopelint-version")" == "v0.0.21" ]] && { echo "  OK --pin wrote .scopelint-version"; ((PASS++)) || true; } || { echo "  FAIL --pin"; ((FAIL++)) || true; }
rm -rf "$PIN_DIR"
echo ""

# --- Error handling ---
echo "== Error handling"
# No version configured: remove .current_version temporarily
mv "$SCOPELINT_DIR/.current_version" "$SCOPELINT_DIR/.current_version.bak" 2>/dev/null || true
out=$(cd /tmp && scopelint --version 2>&1) || true
if [[ "$out" == *"no version configured"* ]]; then
  echo "  OK shim reports 'no version configured' when no default"
  ((PASS++)) || true
else
  echo "  FAIL no version: got $out"
  ((FAIL++)) || true
fi
mv "$SCOPELINT_DIR/.current_version.bak" "$SCOPELINT_DIR/.current_version" 2>/dev/null || true

out=$(SCOPELINT_VERSION=v0.0.99 scopelint --version 2>&1) || true
if [[ "$out" == *"not installed"* ]]; then
  echo "  OK shim reports version not installed for missing version"
  ((PASS++)) || true
else
  echo "  FAIL not installed: got $out"
  ((FAIL++)) || true
fi
echo ""

# --- Summary ---
echo "== Summary"
echo "  Passed: $PASS"
echo "  Failed: $FAIL"
if [[ $FAIL -gt 0 ]]; then
  echo "  SCOPELINT_DIR left at $SCOPELINT_DIR for inspection"
  exit 1
fi
echo "  All tests passed."
exit 0
