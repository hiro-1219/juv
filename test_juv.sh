#!/bin/bash
set -e

# Setup sandbox environment
TEST_DIR=$(mktemp -d -t juv_test_XXXXXXXX)
SANDBOX_DEPOT="$TEST_DIR/julia_depot"
mkdir -p "$SANDBOX_DEPOT"

# Use isolated depot and disable precompile for speed
export JULIA_DEPOT_PATH="$SANDBOX_DEPOT"
export JULIA_PKG_PRECOMPILE_AUTO=0
# We still need the registry, so we copy the current general registry if it exists 
# to avoid long download times, or just let Julia handle it (slower).
# For now, let's just let Julia handle it or use the default one if we can't隔离 completely.
# Actually, the safest way to test is to point JULIA_DEPOT_PATH to a new place.

JUV_BIN="$(pwd)/target/debug/juv"

echo "=== Starting juv Integration Test ==="
echo "Work Dir: $TEST_DIR"
cd "$TEST_DIR"

echo "1. Testing juv init"
"$JUV_BIN" init
if [ ! -f "Project.toml" ]; then
    echo "Error: Project.toml not created"
    exit 1
fi

echo "2. Testing juv add (Registry)"
"$JUV_BIN" add JSON

echo "3. Testing juv add (GitHub)"
"$JUV_BIN" add https://github.com/JuliaLang/Example.jl

echo "4. Testing juv sync-only (Parallel Download)"
# Delete the packages to force a re-download
find "$SANDBOX_DEPOT/packages" -maxdepth 2 -type d -exec rm -rf {} + || true
echo "Cleared packages, re-syncing..."
"$JUV_BIN" sync-only

echo "5. Testing juv run"
echo 'using JSON; using Example; println("JSON and Example loaded successfully")' > test.jl
"$JUV_BIN" run test.jl | grep "JSON and Example loaded successfully"

echo "=== ALL TESTS PASSED ==="

# Cleanup
rm -rf "$TEST_DIR"
