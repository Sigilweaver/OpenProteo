#!/usr/bin/env bash
# truthtest-speclance.sh - run SpecLance's integration tests as the
# stack's truth test.
#
# SpecLance routes all vendor I/O through `openproteo-io` (STRATEGY P0
# #1) and owns the end-to-end coverage that exercises the stack:
# vendor ingest, mzML roundtrip, and Lance store read/write. This
# script is intended to gate umbrella stack releases - see
# `scripts/release-stack.sh --gate-speclance`.
#
# Usage:
#   scripts/truthtest-speclance.sh                       # build + vendors tests
#   scripts/truthtest-speclance.sh --with-corpus         # also corpus tests
#   scripts/truthtest-speclance.sh --no-default-features-only  # smoke build
#
# Flags:
#   --with-corpus               Run with `--all-features`. Requires
#                               `$PROJECTS_DIR/SpecLance/corpus/` to be
#                               present, otherwise exits 2.
#   --no-default-features-only  Only run `cargo build --workspace`,
#                               skip the test phase. Use for fast smoke.
#   --help                      Show this help.
#
# Exit codes:
#   0 green
#   1 unexpected error
#   2 SpecLance sibling repo missing (or --with-corpus and corpus absent)
#   3 build failure
#   4 test failure
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
UMBRELLA_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
PROJECTS_DIR="$(cd "$UMBRELLA_DIR/.." && pwd)"
SPECLANCE_DIR="$PROJECTS_DIR/SpecLance"

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
            echo "truthtest-speclance: unknown flag: $1" >&2
            exit 1
            ;;
    esac
done

if [[ ! -d "$SPECLANCE_DIR" ]]; then
    echo "truthtest-speclance: SpecLance sibling repo not found at $SPECLANCE_DIR" >&2
    exit 2
fi

if [[ $WITH_CORPUS -eq 1 && ! -d "$SPECLANCE_DIR/corpus" ]]; then
    echo "truthtest-speclance: --with-corpus requested but $SPECLANCE_DIR/corpus missing" >&2
    exit 2
fi

echo "truthtest-speclance: using $SPECLANCE_DIR"

cd "$SPECLANCE_DIR"

echo "truthtest-speclance: cargo build --workspace"
if ! cargo build --workspace; then
    echo "truthtest-speclance: build failed" >&2
    exit 3
fi

if [[ $BUILD_ONLY -eq 1 ]]; then
    echo "truthtest-speclance: build-only mode, skipping tests"
    exit 0
fi

if [[ $WITH_CORPUS -eq 1 ]]; then
    echo "truthtest-speclance: cargo test --workspace --all-features"
    if ! cargo test --workspace --all-features; then
        echo "truthtest-speclance: tests failed" >&2
        exit 4
    fi
else
    echo "truthtest-speclance: cargo test -p speclance-ms --features vendors"
    if ! cargo test -p speclance-ms --features vendors; then
        echo "truthtest-speclance: speclance-ms tests failed" >&2
        exit 4
    fi
    echo "truthtest-speclance: cargo test -p speclance-core"
    if ! cargo test -p speclance-core; then
        echo "truthtest-speclance: speclance-core tests failed" >&2
        exit 4
    fi
fi

echo "truthtest-speclance: green"
exit 0
