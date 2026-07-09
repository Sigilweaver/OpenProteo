#!/usr/bin/env bash
# truthtest-prolance.sh - run ProLance's integration tests as the
# stack's truth test.
#
# ProLance routes all vendor I/O through `openproteo-io` (STRATEGY P0
# #1) and owns the end-to-end coverage that exercises the stack:
# vendor ingest, mzML roundtrip, and Lance store read/write. This
# script is intended to gate umbrella stack releases - see
# `scripts/release-stack.sh --gate-prolance`.
#
# Usage:
#   scripts/truthtest-prolance.sh                       # build + vendors tests
#   scripts/truthtest-prolance.sh --with-corpus         # also corpus tests
#   scripts/truthtest-prolance.sh --no-default-features-only  # smoke build
#
# Flags:
#   --with-corpus               Run with `--all-features`. Requires
#                               `$PROJECTS_DIR/ProLance/corpus/` to be
#                               present, otherwise exits 2.
#   --no-default-features-only  Only run `cargo build --workspace`,
#                               skip the test phase. Use for fast smoke.
#   --help                      Show this help.
#
# Exit codes:
#   0 green
#   1 unexpected error
#   2 ProLance sibling repo missing (or --with-corpus and corpus absent)
#   3 build failure
#   4 test failure
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
UMBRELLA_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PROJECTS_DIR="$(cd "$UMBRELLA_DIR/.." && pwd)"
PROLANCE_DIR="$PROJECTS_DIR/ProLance"

WITH_CORPUS=0
BUILD_ONLY=0

while [[ $# -gt 0 ]]; do
    case "$1" in
        --with-corpus)
            WITH_CORPUS=1
            shift
            ;;
        --no-default-features-only)
            BUILD_ONLY=1
            shift
            ;;
        --help|-h)
            sed -n '1,30p' "$0" | sed 's/^# \{0,1\}//'
            exit 0
            ;;
        *)
            echo "truthtest-prolance: unknown flag: $1" >&2
            exit 1
            ;;
    esac
done

if [[ ! -d "$PROLANCE_DIR" ]]; then
    echo "truthtest-prolance: ProLance sibling repo not found at $PROLANCE_DIR" >&2
    exit 2
fi

if [[ $WITH_CORPUS -eq 1 && ! -d "$PROLANCE_DIR/corpus" ]]; then
    echo "truthtest-prolance: --with-corpus requested but $PROLANCE_DIR/corpus missing" >&2
    exit 2
fi

echo "truthtest-prolance: using $PROLANCE_DIR"

cd "$PROLANCE_DIR"

echo "truthtest-prolance: cargo build --workspace"
if ! cargo build --workspace; then
    echo "truthtest-prolance: build failed" >&2
    exit 3
fi

if [[ $BUILD_ONLY -eq 1 ]]; then
    echo "truthtest-prolance: build-only mode, skipping tests"
    exit 0
fi

if [[ $WITH_CORPUS -eq 1 ]]; then
    echo "truthtest-prolance: cargo test --workspace --all-features"
    if ! cargo test --workspace --all-features; then
        echo "truthtest-prolance: tests failed" >&2
        exit 4
    fi
else
    echo "truthtest-prolance: cargo test -p prolance-ms --features vendors"
    if ! cargo test -p prolance-ms --features vendors; then
        echo "truthtest-prolance: prolance-ms tests failed" >&2
        exit 4
    fi
    echo "truthtest-prolance: cargo test -p prolance-core"
    if ! cargo test -p prolance-core; then
        echo "truthtest-prolance: prolance-core tests failed" >&2
        exit 4
    fi
fi

echo "truthtest-prolance: green"
exit 0
