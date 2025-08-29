#!/bin/bash
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Set strict environment
export CARGO_TERM_COLOR=always
export RUST_BACKTRACE=1
export RUSTFLAGS="-D warnings"
export CARGO_INCREMENTAL=0
export RUSTDOCFLAGS="-D warnings"

echo -e "${GREEN}Running warcraft-rs QA Suite...${NC}"

# Function to run a check
run_check() {
    local name="$1"
    shift
    echo -e "${YELLOW}► $name${NC}"
    if "$@"; then
        echo -e "${GREEN}  ✓ $name passed${NC}"
    else
        echo -e "${RED}  ✗ $name failed${NC}"
        exit 1
    fi
}

# Core checks
run_check "Format Check" cargo fmt --all -- --check
run_check "Compilation (all features)" cargo check --all-features --all-targets
run_check "Compilation (no features)" cargo check --no-default-features --all-targets
run_check "Clippy (strict)" cargo clippy --workspace --all-targets --all-features -- \
    -D warnings

run_check "Tests (all features)" cargo test --all-features --workspace
run_check "Tests (no features)" cargo test --no-default-features --workspace
run_check "Tests (release mode)" cargo test --release --all-features --workspace
run_check "Documentation" cargo doc --all-features --workspace --no-deps

# Optional advanced checks (if tools are installed)
if command -v cargo-deny &> /dev/null; then
    run_check "Dependency Audit" cargo deny check
elif command -v cargo-audit &> /dev/null; then
    run_check "Security Audit" cargo audit
fi

if command -v cargo-outdated &> /dev/null; then
    echo -e "${YELLOW}► Outdated Dependencies${NC}"
    cargo outdated --workspace --depth 1 || true
    echo -e "${GREEN}  ✓ Outdated check complete (informational)${NC}"
fi

if command -v cargo-udeps &> /dev/null; then
    run_check "Unused Dependencies" cargo +nightly udeps --all-targets --all-features
fi

# Individual crate tests
echo -e "${YELLOW}► Individual Crate Tests${NC}"
for crate in storm-ffi wow-mpq wow-cdbc wow-blp wow-m2 wow-wmo wow-adt wow-wdl wow-wdt warcraft-rs; do
    run_check "  Testing $crate" cargo test -p "$crate" --all-features
done

# Release build
run_check "Release Build" cargo build --release --workspace

# Check for workspace members
echo -e "${YELLOW}► Workspace Structure${NC}"
MEMBER_COUNT=$(cargo metadata --format-version 1 --no-deps | jq '.workspace_members | length')
echo -e "  Found $MEMBER_COUNT workspace member(s)"
echo -e "${GREEN}  ✓ Workspace structure verified${NC}"

echo -e "${GREEN}✅ All QA checks passed!${NC}"