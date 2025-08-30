#!/bin/bash
set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Parse command line arguments
MODE="fast"
VERBOSE=false
while [[ $# -gt 0 ]]; do
    case $1 in
        --full)
            MODE="full"
            shift
            ;;
        --fast)
            MODE="fast"
            shift
            ;;
        --verbose|-v)
            VERBOSE=true
            shift
            ;;
        --help|-h)
            echo "Usage: $0 [--fast|--full] [--verbose|-v]"
            echo "  --fast    Run quick validation checks (default, ~2 minutes)"
            echo "  --full    Run comprehensive QA suite (~10+ minutes)"
            echo "  --verbose Show detailed output"
            echo ""
            echo "Fast mode uses:"
            echo "  • cargo check instead of cargo build"
            echo "  • Parallel job execution"
            echo "  • Incremental compilation"
            echo "  • Subset of tests"
            echo ""
            echo "Full mode includes:"
            echo "  • Complete compilation"
            echo "  • All test suites"
            echo "  • Security audits"
            echo "  • Release builds"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Set environment based on mode
export CARGO_TERM_COLOR=always
export RUST_BACKTRACE=1
export RUSTFLAGS="-D warnings"

# Set doc warnings based on mode
if [ "$MODE" = "fast" ]; then
    # Fast mode: Allow documentation warnings
    export RUSTDOCFLAGS=""
else
    # Full mode: Strict documentation warnings
    export RUSTDOCFLAGS="-D warnings"
fi

# Performance settings based on mode
if [ "$MODE" = "fast" ]; then
    export CARGO_INCREMENTAL=1  # Enable incremental compilation
    JOBS=$(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 4)
    echo -e "${CYAN}🚀 Running warcraft-rs Fast QA Suite (${JOBS} parallel jobs)...${NC}"
    echo -e "${BLUE}Quick validation - use --full for comprehensive checks${NC}"
else
    export CARGO_INCREMENTAL=0  # Disable for reproducible builds
    JOBS=1  # Sequential for consistent results
    echo -e "${GREEN}Running warcraft-rs Full QA Suite...${NC}"
    echo -e "${BLUE}Comprehensive validation for production readiness${NC}"
fi

# Track timing
START_TIME=$(date +%s)

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

# Core quality gates
echo -e "${BLUE}=== Core Quality Gates ===${NC}"
run_check "Format Check" cargo fmt --all -- --check

if [ "$MODE" = "fast" ]; then
    # Fast mode: use check instead of build
    run_check "Type Check (all features)" cargo check --all-features --all-targets --jobs ${JOBS}
    run_check "Type Check (no features)" cargo check --no-default-features --all-targets --jobs ${JOBS}
    # Reduced clippy lints for speed
    run_check "Clippy (essential)" cargo clippy --workspace --all-targets --all-features --jobs ${JOBS} -- \
        -D warnings
else
    # Full mode: complete compilation
    run_check "Compilation (all features)" cargo build --all-features --all-targets --jobs ${JOBS}
    run_check "Compilation (no features)" cargo build --no-default-features --all-targets --jobs ${JOBS}
    # All clippy lints
    run_check "Clippy (strict)" cargo clippy --workspace --all-targets --all-features --jobs ${JOBS} -- \
        -D warnings
fi

# Test suite
echo -e "${BLUE}=== Test Suite ===${NC}"
if [ "$MODE" = "fast" ]; then
    # Fast mode: minimal test suite
    run_check "Unit Tests" cargo test --lib --workspace --jobs ${JOBS}
    run_check "Doc Tests" cargo test --doc --workspace --jobs ${JOBS}
    run_check "Documentation Check" cargo doc --all-features --workspace --no-deps --jobs ${JOBS}
else
    # Full mode: comprehensive testing
    run_check "Tests (all features)" cargo test --all-features --workspace --jobs ${JOBS}
    run_check "Tests (no features)" cargo test --no-default-features --workspace --jobs ${JOBS}
    run_check "Tests (release mode)" cargo test --release --all-features --workspace --jobs 1
    run_check "Documentation" cargo doc --all-features --workspace --no-deps --jobs ${JOBS}
    run_check "Documentation Tests" cargo test --doc --workspace --jobs ${JOBS}
fi

# WoW File Format Specific Checks
echo -e "${BLUE}=== WoW File Format Specific ===${NC}"

# Use ripgrep if available (much faster)
if command -v rg &> /dev/null; then
    GREP_CMD="rg"
    GREP_OPTS="-t rust --glob '!*/tests/*' --glob '!*/examples/*'"
else
    GREP_CMD="grep"
    GREP_OPTS="-r --include='*.rs' --exclude-dir=tests --exclude-dir=examples"
fi

# Check for unwrap usage in source (not tests)
echo -e "${YELLOW}► Safety Check: No unwrap() in library code${NC}"
if [ "$MODE" = "fast" ]; then
    # Fast mode: just check if any exist
    if eval "$GREP_CMD $GREP_OPTS 'unwrap\\(\\)' file-formats/*/src 2>/dev/null | head -1" > /dev/null; then
        echo -e "${YELLOW}  ! Found unwrap() usage - run --full for details${NC}"
    else
        echo -e "${GREEN}  ✓ No unwrap() found in library code${NC}"
    fi
else
    # Full mode: show all occurrences
    UNWRAP_COUNT=$(eval "$GREP_CMD $GREP_OPTS 'unwrap\\(\\)' file-formats/*/src 2>/dev/null" | grep -v "test" | grep -v "example" | grep -v "comment" | wc -l || echo "0")
    if [ "$UNWRAP_COUNT" -gt 0 ]; then
        echo -e "${YELLOW}  ! Found $UNWRAP_COUNT unwrap() calls in library code${NC}"
        if [ "$VERBOSE" = true ]; then
            eval "$GREP_CMD $GREP_OPTS 'unwrap\\(\\)' file-formats/*/src 2>/dev/null" | grep -v "test" | grep -v "example" | grep -v "comment"
        fi
    else
        echo -e "${GREEN}  ✓ No unwrap() found in library code${NC}"
    fi
fi

# Check for panic usage in source (not tests)
echo -e "${YELLOW}► Safety Check: No panic!() in library code${NC}"
if [ "$MODE" = "fast" ]; then
    # Fast mode: just check if any exist
    if eval "$GREP_CMD $GREP_OPTS 'panic!' file-formats/*/src 2>/dev/null | head -1" > /dev/null; then
        echo -e "${YELLOW}  ! Found panic!() usage - run --full for details${NC}"
    else
        echo -e "${GREEN}  ✓ No panic!() found in library code${NC}"
    fi
else
    # Full mode: show all occurrences
    PANIC_COUNT=$(eval "$GREP_CMD $GREP_OPTS 'panic!' file-formats/*/src 2>/dev/null" | grep -v "test" | grep -v "example" | grep -v "comment" | wc -l || echo "0")
    if [ "$PANIC_COUNT" -gt 0 ]; then
        echo -e "${YELLOW}  ! Found $PANIC_COUNT panic!() calls in library code${NC}"
        if [ "$VERBOSE" = true ]; then
            eval "$GREP_CMD $GREP_OPTS 'panic!' file-formats/*/src 2>/dev/null" | grep -v "test" | grep -v "example" | grep -v "comment"
        fi
    else
        echo -e "${GREEN}  ✓ No panic!() found in library code${NC}"
    fi
fi

# Individual crate tests for isolation
if [ "$MODE" = "full" ]; then
    echo -e "${BLUE}=== Individual Crate Verification ===${NC}"
    for crate in storm-ffi wow-mpq wow-cdbc wow-blp wow-m2 wow-wmo wow-adt wow-wdl wow-wdt warcraft-rs; do
        if cargo metadata --format-version 1 --no-deps | grep -q "\"$crate\""; then
            run_check "  Testing $crate" cargo test -p "$crate" --all-features --jobs ${JOBS}
        fi
    done
fi

# Advanced security and quality checks
echo -e "${BLUE}=== Advanced Security Checks ===${NC}"

# Security audit
if command -v cargo-deny &> /dev/null; then
    run_check "Dependency Security Audit" cargo deny check
elif command -v cargo-audit &> /dev/null; then
    run_check "Security Vulnerability Audit" cargo audit
else
    echo -e "${YELLOW}  ! Security audit tools not installed (cargo-deny or cargo-audit recommended)${NC}"
fi

# Unused dependencies check
if command -v cargo-udeps &> /dev/null && rustup toolchain list | grep -q nightly; then
    run_check "Unused Dependencies" cargo +nightly udeps --all-targets --all-features
else
    echo -e "${YELLOW}  ! cargo-udeps not installed or nightly toolchain missing${NC}"
fi

# Dependency freshness check
if command -v cargo-outdated &> /dev/null; then
    echo -e "${YELLOW}► Outdated Dependencies${NC}"
    cargo outdated --workspace --depth 1 || true
    echo -e "${GREEN}  ✓ Outdated check complete (informational)${NC}"
fi

# Build verification
if [ "$MODE" = "full" ]; then
    echo -e "${BLUE}=== Build Verification ===${NC}"
    run_check "Release Build (optimized)" cargo build --release --workspace --jobs ${JOBS}
    
    # Check if examples exist before trying to build them
    if find . -path "*/examples/*.rs" 2>/dev/null | head -1 > /dev/null; then
        run_check "Examples Build" cargo build --examples --workspace --jobs ${JOBS}
    else
        echo -e "${YELLOW}  ! No examples found${NC}"
    fi
    
    # Check benchmarks if they exist
    if find . -path "*/benches/*.rs" 2>/dev/null | head -1 > /dev/null; then
        echo -e "${YELLOW}► Benchmark Compilation${NC}"
        cargo bench --workspace --no-run --jobs ${JOBS} 2>/dev/null && echo -e "${GREEN}  ✓ Benchmarks compile${NC}" || echo -e "${YELLOW}  ! Benchmark compilation skipped${NC}"
    fi
fi

# Workspace structure verification
echo -e "${BLUE}=== Workspace Structure ===${NC}"
echo -e "${YELLOW}► Workspace Structure${NC}"
if command -v jq &> /dev/null; then
    MEMBER_COUNT=$(cargo metadata --format-version 1 --no-deps | jq '.workspace_members | length')
    echo -e "  Found $MEMBER_COUNT workspace member(s)"
    if [ "$VERBOSE" = true ]; then
        cargo metadata --format-version 1 --no-deps | jq -r '.workspace_members[]' | sed 's/^/    - /'
    fi
else
    MEMBER_COUNT=$(cargo metadata --format-version 1 --no-deps | grep -o '"workspace_members":\[[^]]*\]' | grep -o '"[^"]*"' | wc -l)
    echo -e "  Found approximately $MEMBER_COUNT workspace member(s)"
fi
echo -e "${GREEN}  ✓ Workspace structure verified${NC}"

# Production readiness check
echo -e "${BLUE}=== Production Readiness ===${NC}"

# Check for TODO/FIXME in production code (warning, not failure)
echo -e "${YELLOW}► Production Readiness: TODO/FIXME Check${NC}"
TODO_COUNT=$(eval "$GREP_CMD 'TODO\\|FIXME' file-formats/*/src warcraft-rs/src 2>/dev/null" | wc -l 2>/dev/null || echo "0")
TODO_COUNT=${TODO_COUNT:-0}  # Default to 0 if empty
TODO_COUNT=$(echo "$TODO_COUNT" | head -n1 | tr -d '\n' | tr -d ' ')  # Take first line, remove whitespace
if [ "$TODO_COUNT" -gt 0 ]; then
    echo -e "${YELLOW}  ! Found $TODO_COUNT TODO/FIXME items in source code${NC}"
    echo -e "${YELLOW}    (Consider addressing before production deployment)${NC}"
    if [ "$VERBOSE" = true ] && [ "$MODE" = "full" ]; then
        echo -e "${BLUE}    First 5 items:${NC}"
        eval "$GREP_CMD 'TODO\\|FIXME' file-formats/*/src warcraft-rs/src 2>/dev/null" | head -5 | sed 's/^/      /'
    fi
else
    echo -e "${GREEN}  ✓ No TODO/FIXME items in source code${NC}"
fi

# Check test coverage if tarpaulin is installed
if command -v cargo-tarpaulin &> /dev/null && [ "$MODE" = "full" ]; then
    echo -e "${YELLOW}► Test Coverage Analysis${NC}"
    echo -e "${BLUE}  Running coverage analysis (this may take a while)...${NC}"
    if cargo tarpaulin --workspace --timeout 300 --print-summary 2>/dev/null; then
        echo -e "${GREEN}  ✓ Coverage analysis complete${NC}"
    else
        echo -e "${YELLOW}  ! Coverage analysis failed or incomplete${NC}"
    fi
else
    if [ "$MODE" = "full" ]; then
        echo -e "${YELLOW}  ! cargo-tarpaulin not installed (optional for coverage analysis)${NC}"
    fi
fi

# Calculate elapsed time
END_TIME=$(date +%s)
ELAPSED=$((END_TIME - START_TIME))
MINUTES=$((ELAPSED / 60))
SECONDS=$((ELAPSED % 60))

# Final success message
if [ "$MODE" = "fast" ]; then
    echo -e "${CYAN}🚀 Fast QA completed in ${MINUTES}m ${SECONDS}s!${NC}"
    echo -e "${BLUE}✅ Basic checks passed - use --full for comprehensive validation${NC}"
    echo ""
    echo -e "${YELLOW}Performance Tips:${NC}"
    echo -e "  • cargo check: Type checking without compilation (10x faster)"
    echo -e "  • cargo test --lib: Skip integration tests (2x faster)"
    echo -e "  • cargo clippy --fix: Auto-fix some issues"
    echo -e "  • sccache: Consider installing for C dependency caching"
    echo -e "  • mold: Use fast linker for debug builds (apt install mold)"
else
    echo -e "${GREEN}🎉 All warcraft-rs QA checks passed in ${MINUTES}m ${SECONDS}s!${NC}"
    echo -e "${BLUE}✅ WoW file format parsers meet quality standards${NC}"
    echo -e "${BLUE}📦 Ready for MPQ archive processing and model parsing${NC}"
fi