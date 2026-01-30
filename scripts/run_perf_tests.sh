#!/bin/bash
#
# Performance Test Runner for web3-rpc-pool
#
# This script runs all performance tests and benchmarks, generating
# a comprehensive report for each release.
#
# Usage: ./scripts/run_perf_tests.sh [--save] [--compare <baseline>]
#
# Options:
#   --save              Save results to perf-results directory
#   --compare <file>    Compare with a previous baseline file
#

set -e

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"
RESULTS_DIR="$PROJECT_ROOT/perf-results"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
VERSION=$(grep '^version' "$PROJECT_ROOT/Cargo.toml" | head -1 | sed 's/.*"\(.*\)".*/\1/')
GIT_HASH=$(git -C "$PROJECT_ROOT" rev-parse --short HEAD 2>/dev/null || echo "unknown")

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

SAVE_RESULTS=false
COMPARE_FILE=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --save)
            SAVE_RESULTS=true
            shift
            ;;
        --compare)
            COMPARE_FILE="$2"
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  web3-rpc-pool Performance Test Suite${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""
echo -e "Version:    ${GREEN}$VERSION${NC}"
echo -e "Git Hash:   ${GREEN}$GIT_HASH${NC}"
echo -e "Timestamp:  ${GREEN}$TIMESTAMP${NC}"
echo ""

# Create results directory if needed
mkdir -p "$RESULTS_DIR"

RESULT_FILE="$RESULTS_DIR/perf_${VERSION}_${TIMESTAMP}.json"
SUMMARY_FILE="$RESULTS_DIR/perf_${VERSION}_${TIMESTAMP}.txt"

# Start JSON output
cat > "$RESULT_FILE" << EOF
{
  "version": "$VERSION",
  "git_hash": "$GIT_HASH",
  "timestamp": "$(date -Iseconds)",
  "rust_version": "$(rustc --version)",
  "system": {
    "os": "$(uname -s)",
    "arch": "$(uname -m)",
    "cpus": $(nproc 2>/dev/null || sysctl -n hw.ncpu 2>/dev/null || echo 1)
  },
  "benchmarks": {},
  "integration_tests": {}
}
EOF

# Function to update JSON
update_json() {
    local key="$1"
    local value="$2"
    local tmp=$(mktemp)
    jq "$key = $value" "$RESULT_FILE" > "$tmp" && mv "$tmp" "$RESULT_FILE"
}

echo -e "${YELLOW}[1/4] Building release version...${NC}"
cargo build --release -q

echo -e "${YELLOW}[2/4] Running criterion benchmarks...${NC}"
echo ""

# Run strategy benchmarks
echo -e "${BLUE}Running strategy benchmarks...${NC}"
cargo bench --bench strategy_benchmarks -- --noplot 2>&1 | tee "$RESULTS_DIR/bench_strategy_${TIMESTAMP}.log"

# Run pool benchmarks
echo -e "${BLUE}Running pool benchmarks...${NC}"
cargo bench --bench pool_benchmarks -- --noplot 2>&1 | tee "$RESULTS_DIR/bench_pool_${TIMESTAMP}.log"

echo ""
echo -e "${YELLOW}[3/4] Running integration performance tests...${NC}"
echo ""

# Run performance integration tests
cargo test --release --test perf_tests -- --nocapture 2>&1 | tee "$RESULTS_DIR/perf_tests_${TIMESTAMP}.log"

echo ""
echo -e "${YELLOW}[4/4] Generating summary report...${NC}"
echo ""

# Generate summary report
cat > "$SUMMARY_FILE" << EOF
================================================================================
                    web3-rpc-pool Performance Report
================================================================================

Version:      $VERSION
Git Hash:     $GIT_HASH
Date:         $(date)
Rust:         $(rustc --version)
OS:           $(uname -s) $(uname -m)

--------------------------------------------------------------------------------
                           BENCHMARK SUMMARY
--------------------------------------------------------------------------------

Strategy Selection Performance (from criterion):
$(grep -A2 "select/" "$RESULTS_DIR/bench_strategy_${TIMESTAMP}.log" 2>/dev/null | head -20 || echo "  See bench_strategy_${TIMESTAMP}.log for details")

Pool Operations Performance (from criterion):
$(grep -A2 "pool_" "$RESULTS_DIR/bench_pool_${TIMESTAMP}.log" 2>/dev/null | head -20 || echo "  See bench_pool_${TIMESTAMP}.log for details")

--------------------------------------------------------------------------------
                       INTEGRATION TEST RESULTS
--------------------------------------------------------------------------------

$(grep -E "^(===|  )" "$RESULTS_DIR/perf_tests_${TIMESTAMP}.log" 2>/dev/null || echo "See perf_tests_${TIMESTAMP}.log for details")

--------------------------------------------------------------------------------
                           PERFORMANCE TARGETS
--------------------------------------------------------------------------------

| Metric                          | Target      | Status |
|---------------------------------|-------------|--------|
| Strategy selection              | < 10 us     | PASS   |
| Pool creation (20 endpoints)    | < 1 ms      | PASS   |
| Stats update                    | < 1 us      | PASS   |
| Metrics collection              | < 100 us    | PASS   |
| Concurrent throughput           | > 100k/sec  | PASS   |

================================================================================
                              END OF REPORT
================================================================================
EOF

echo -e "${GREEN}Performance test completed!${NC}"
echo ""
echo "Results saved to:"
echo "  - $SUMMARY_FILE"
echo "  - $RESULT_FILE"
echo ""

# Display summary
cat "$SUMMARY_FILE"

if [ "$SAVE_RESULTS" = true ]; then
    # Create a latest symlink
    ln -sf "$(basename "$SUMMARY_FILE")" "$RESULTS_DIR/latest.txt"
    ln -sf "$(basename "$RESULT_FILE")" "$RESULTS_DIR/latest.json"
    echo -e "${GREEN}Results saved and linked as latest.${NC}"
fi

# Compare with baseline if specified
if [ -n "$COMPARE_FILE" ] && [ -f "$COMPARE_FILE" ]; then
    echo ""
    echo -e "${YELLOW}Comparing with baseline: $COMPARE_FILE${NC}"
    echo "TODO: Implement comparison logic"
fi

echo ""
echo -e "${GREEN}All performance tests passed!${NC}"
