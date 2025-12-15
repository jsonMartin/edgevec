#!/bin/bash
# EdgeVec Pre-Release Validation Script
# Version: 1.1.0
# Created: 2025-12-15 (W18.1)
# Updated: 2025-12-15 (W18.1 hostile review fixes)
#
# Run this before any release to catch CI issues locally.
# This script validates Phases 1-4 of the release checklist.
#
# Addresses hostile review findings:
# - C1: cargo publish --dry-run
# - C2: npm publish --dry-run
# - Week 17 post-mortem lessons
# - M5: Removed set -e to allow graceful error tracking
#
# Platform Support:
# - Linux/macOS: Native execution
# - Windows: Requires WSL (Windows Subsystem for Linux)
#   Install WSL: https://docs.microsoft.com/en-us/windows/wsl/install

# NOTE: We intentionally do NOT use 'set -e' here.
# The check_result function tracks failures gracefully and provides
# a summary at the end. Using 'set -e' would cause immediate exit
# on any failure, preventing the full validation report.

echo "=========================================="
echo "  EdgeVec Pre-Release Validation Script"
echo "=========================================="
echo ""

# Colors for output (with fallback for non-color terminals)
if [ -t 1 ] && [ "$(tput colors 2>/dev/null || echo 0)" -ge 8 ]; then
    RED='\033[0;31m'
    GREEN='\033[0;32m'
    YELLOW='\033[1;33m'
    NC='\033[0m' # No Color
else
    RED=''
    GREEN=''
    YELLOW=''
    NC=''
fi

# Track failures
FAILURES=0

echo "=== Phase 0: Prerequisites ==="
echo ""

# Check required tools
echo "0.1 Checking required tools..."
MISSING_TOOLS=0
for tool in cargo npm git; do
    if ! command -v $tool &> /dev/null; then
        echo -e "${RED}MISSING${NC}: $tool"
        MISSING_TOOLS=$((MISSING_TOOLS + 1))
    fi
done
if [ $MISSING_TOOLS -gt 0 ]; then
    echo -e "${RED}FAIL${NC}: Missing $MISSING_TOOLS required tool(s)"
    echo "  Install missing tools before continuing."
    exit 1
else
    echo -e "${GREEN}PASS${NC}: All required tools present"
fi

# Check version sync [m2]
echo ""
echo "0.2 Checking version synchronization..."
CARGO_VERSION=$(grep -m1 '^version' Cargo.toml | sed 's/.*"\(.*\)".*/\1/')
if [ -f "pkg/package.json" ]; then
    NPM_VERSION=$(grep -m1 '"version"' pkg/package.json | sed 's/.*"\([0-9.]*\)".*/\1/')
    if [ "$CARGO_VERSION" = "$NPM_VERSION" ]; then
        echo -e "${GREEN}PASS${NC}: Cargo.toml ($CARGO_VERSION) matches pkg/package.json ($NPM_VERSION)"
    else
        echo -e "${YELLOW}WARN${NC}: Version mismatch - Cargo.toml ($CARGO_VERSION) != pkg/package.json ($NPM_VERSION)"
        echo "  Ensure versions are synchronized before release."
    fi
else
    echo -e "${YELLOW}INFO${NC}: pkg/package.json not found (will be created by wasm-pack)"
fi

echo ""

# Helper function to check command result
# Usage: run_check "description" command args...
run_check() {
    local description="$1"
    shift
    if "$@"; then
        echo -e "${GREEN}PASS${NC}: $description"
        return 0
    else
        echo -e "${RED}FAIL${NC}: $description"
        FAILURES=$((FAILURES + 1))
        return 1
    fi
}

# Legacy helper for backward compatibility (checks $? after command)
check_result() {
    local exit_code=$?
    if [ $exit_code -eq 0 ]; then
        echo -e "${GREEN}PASS${NC}: $1"
    else
        echo -e "${RED}FAIL${NC}: $1"
        FAILURES=$((FAILURES + 1))
    fi
}

echo "=== Phase 1: Local Code Quality ==="
echo ""

echo "1.1 Checking formatting..."
if cargo fmt -- --check; then
    echo -e "${GREEN}PASS${NC}: cargo fmt -- --check"
else
    echo -e "${RED}FAIL${NC}: cargo fmt -- --check"
    FAILURES=$((FAILURES + 1))
fi

echo ""
echo "1.2 Running Clippy (all targets)..."
if cargo clippy --all-targets -- -D clippy::correctness -W clippy::suspicious -W clippy::style; then
    echo -e "${GREEN}PASS${NC}: cargo clippy --all-targets"
else
    echo -e "${RED}FAIL${NC}: cargo clippy --all-targets"
    FAILURES=$((FAILURES + 1))
fi

echo ""
echo "1.3 Checking documentation..."
DOC_WARNINGS=$(cargo doc --no-deps 2>&1 | grep -c "warning" || true)
if [ "$DOC_WARNINGS" -eq 0 ]; then
    echo -e "${GREEN}PASS${NC}: Documentation clean (no warnings)"
else
    echo -e "${YELLOW}WARN${NC}: Documentation has $DOC_WARNINGS warnings"
fi

echo ""
echo "=== Phase 2: CI Simulation ==="
echo ""
echo "Setting CI environment variables..."
export RUSTFLAGS="-C target-cpu=x86-64-v2"
export PROPTEST_CASES=32
export NUM_VECTORS=1000

echo "  RUSTFLAGS=$RUSTFLAGS"
echo "  PROPTEST_CASES=$PROPTEST_CASES"
echo "  NUM_VECTORS=$NUM_VECTORS"
echo ""

echo "2.1 Running test suite (CI simulation)..."
START_TIME=$(date +%s)
if cargo test --all; then
    TEST_PASSED=true
else
    TEST_PASSED=false
fi
END_TIME=$(date +%s)
TEST_DURATION=$((END_TIME - START_TIME))

if [ "$TEST_PASSED" = true ]; then
    echo -e "${GREEN}PASS${NC}: cargo test --all"
else
    echo -e "${RED}FAIL${NC}: cargo test --all"
    FAILURES=$((FAILURES + 1))
fi

echo ""
echo "Test suite completed in ${TEST_DURATION} seconds"

if [ $TEST_DURATION -gt 900 ]; then
    echo -e "${RED}FAIL${NC}: Test suite took > 15 minutes ($TEST_DURATION seconds)"
    FAILURES=$((FAILURES + 1))
else
    echo -e "${GREEN}PASS${NC}: Test suite completed in < 15 minutes"
fi

echo ""
echo "=== Phase 3: WASM Validation ==="
echo ""

echo "3.1 Checking WASM target compilation..."
if cargo check --target wasm32-unknown-unknown; then
    echo -e "${GREEN}PASS${NC}: cargo check --target wasm32-unknown-unknown"
else
    echo -e "${RED}FAIL${NC}: cargo check --target wasm32-unknown-unknown"
    FAILURES=$((FAILURES + 1))
fi

echo ""
echo "3.2 Building WASM package..."
if ! command -v wasm-pack &> /dev/null; then
    echo -e "${RED}FAIL${NC}: wasm-pack not installed"
    echo ""
    echo "  wasm-pack is REQUIRED for releases. Install it with:"
    echo "    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh"
    echo "  Or: cargo install wasm-pack"
    FAILURES=$((FAILURES + 1))
else
    if wasm-pack build --release; then
        echo -e "${GREEN}PASS${NC}: wasm-pack build --release"
    else
        echo -e "${RED}FAIL${NC}: wasm-pack build --release"
        FAILURES=$((FAILURES + 1))
    fi

    echo ""
    echo "3.3 Checking bundle size..."
    if [ -f "pkg/edgevec_bg.wasm" ]; then
        WASM_SIZE=$(wc -c < pkg/edgevec_bg.wasm)
        WASM_SIZE_KB=$((WASM_SIZE / 1024))

        # 500KB = 500 * 1024 = 512,000 bytes
        MAX_BUNDLE_SIZE=512000
        if [ $WASM_SIZE -lt $MAX_BUNDLE_SIZE ]; then
            echo -e "${GREEN}PASS${NC}: Bundle size is ${WASM_SIZE_KB}KB (< 500KB limit)"
        else
            echo -e "${RED}FAIL${NC}: Bundle size is ${WASM_SIZE_KB}KB (exceeds 500KB limit)"
            FAILURES=$((FAILURES + 1))
        fi
    else
        echo -e "${YELLOW}WARN${NC}: pkg/edgevec_bg.wasm not found (wasm-pack build may have failed)"
    fi
fi

echo ""
echo "=== Phase 4: Dry Run ==="
echo ""

echo "4.1 [C1 FIX] Cargo publish dry-run..."
if cargo publish --dry-run; then
    echo -e "${GREEN}PASS${NC}: cargo publish --dry-run"
else
    echo -e "${RED}FAIL${NC}: cargo publish --dry-run"
    FAILURES=$((FAILURES + 1))
fi

echo ""
echo "4.2 [C2 FIX] NPM pack dry-run..."
if [ -d "pkg" ]; then
    cd pkg
    npm pack --dry-run
    PACK_RESULT=$?
    cd ..
    if [ $PACK_RESULT -eq 0 ]; then
        echo -e "${GREEN}PASS${NC}: npm pack --dry-run"
    else
        echo -e "${RED}FAIL${NC}: npm pack --dry-run"
        FAILURES=$((FAILURES + 1))
    fi
else
    echo -e "${YELLOW}SKIP${NC}: pkg/ directory not found"
fi

echo ""
echo "=========================================="
echo "  Pre-Release Validation Complete"
echo "=========================================="
echo ""

if [ $FAILURES -eq 0 ]; then
    echo -e "${GREEN}SUCCESS${NC}: All checks passed!"
    echo ""
    echo "Safe to proceed with release."
    echo ""
    echo "Next steps:"
    echo "  1. Create release branch: git checkout -b release/vX.Y.Z"
    echo "  2. Update version numbers in Cargo.toml and pkg/package.json"
    echo "  3. Update CHANGELOG.md"
    echo "  4. Commit: git commit -am 'chore: bump version to vX.Y.Z'"
    echo "  5. Push: git push -u origin release/vX.Y.Z"
    echo "  6. Wait for CI green"
    echo "  7. Merge to main and tag"
    echo "  8. Publish: cargo publish && cd pkg && npm publish"
    echo ""
    echo "See docs/RELEASE_CHECKLIST.md for full instructions."
    exit 0
else
    echo -e "${RED}FAILURE${NC}: $FAILURES check(s) failed!"
    echo ""
    echo "DO NOT proceed with release until all checks pass."
    echo ""
    echo "Recovery steps by failure type:"
    echo "  - Format failed:  cargo fmt"
    echo "  - Clippy failed:  cargo clippy --fix --allow-dirty"
    echo "  - Tests failed:   cargo test --no-fail-fast 2>&1 | head -100"
    echo "  - WASM failed:    rustup target add wasm32-unknown-unknown"
    echo "  - wasm-pack:      cargo install wasm-pack"
    echo "  - Publish failed: Check Cargo.toml [package] section"
    echo ""
    echo "See docs/RELEASE_CHECKLIST.md for detailed troubleshooting."
    exit 1
fi
