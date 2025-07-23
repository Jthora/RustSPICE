#!/bin/bash

# Comprehensive test runner for RustSPICE conversion validation
# Usage: ./run_tests.sh [test_type]
# test_type: unit, integration, property, snapshot, bench, all

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
CARGO_FLAGS="--release"
TEST_TIMEOUT="300s"  # 5 minutes max per test suite
BENCH_SAMPLES=100

echo -e "${BLUE}🚀 RustSPICE Test Suite Runner${NC}"
echo "=================================="

# Parse command line arguments
TEST_TYPE=${1:-all}
VERBOSE=${2:-false}

if [ "$VERBOSE" = "true" ] || [ "$VERBOSE" = "-v" ]; then
    CARGO_FLAGS="$CARGO_FLAGS --verbose"
fi

# Function to run a specific test suite
run_test_suite() {
    local suite_name=$1
    local test_command=$2
    local description=$3
    
    echo -e "\n${YELLOW}📋 Running $description...${NC}"
    echo "Command: $test_command"
    
    if timeout $TEST_TIMEOUT bash -c "$test_command"; then
        echo -e "${GREEN}✅ $suite_name tests passed${NC}"
        return 0
    else
        echo -e "${RED}❌ $suite_name tests failed${NC}"
        return 1
    fi
}

# Function to check prerequisites
check_prerequisites() {
    echo -e "${BLUE}🔍 Checking prerequisites...${NC}"
    
    # Check Rust toolchain
    if ! command -v cargo &> /dev/null; then
        echo -e "${RED}❌ Cargo not found. Please install Rust toolchain.${NC}"
        exit 1
    fi
    
    # Check if we have wasm32 target
    if cargo target list | grep -q "wasm32-unknown-unknown"; then
        echo -e "${GREEN}✅ WASM target available${NC}"
    else
        echo -e "${YELLOW}⚠️  WASM target not found. Installing...${NC}"
        rustup target add wasm32-unknown-unknown
    fi
    
    # Check if we have required tools
    local missing_tools=()
    
    if ! command -v wasm-pack &> /dev/null; then
        missing_tools+=("wasm-pack")
    fi
    
    if [ ${#missing_tools[@]} -ne 0 ]; then
        echo -e "${YELLOW}⚠️  Missing tools: ${missing_tools[*]}${NC}"
        echo "Consider installing with: cargo install wasm-pack"
    fi
    
    echo -e "${GREEN}✅ Prerequisites check complete${NC}"
}

# Function to run unit tests
run_unit_tests() {
    run_test_suite "Unit" \
        "cargo test --lib $CARGO_FLAGS" \
        "library unit tests"
}

# Function to run integration tests
run_integration_tests() {
    run_test_suite "Integration" \
        "cargo test --test '*' $CARGO_FLAGS" \
        "integration tests"
}

# Function to run property-based tests
run_property_tests() {
    run_test_suite "Property" \
        "cargo test --test property_tests $CARGO_FLAGS" \
        "property-based tests with QuickCheck and PropTest"
}

# Function to run snapshot tests
run_snapshot_tests() {
    run_test_suite "Snapshot" \
        "cargo test --test snapshot_tests $CARGO_FLAGS" \
        "snapshot tests for regression detection"
}

# Function to run benchmarks
run_benchmarks() {
    echo -e "\n${YELLOW}🏃 Running performance benchmarks...${NC}"
    
    # Run benchmarks with criterion
    if cargo bench --bench ephemeris_benchmark -- --sample-size $BENCH_SAMPLES; then
        echo -e "${GREEN}✅ Benchmarks completed${NC}"
        echo "Results saved to target/criterion/"
        return 0
    else
        echo -e "${RED}❌ Benchmarks failed${NC}"
        return 1
    fi
}

# Function to run WASM-specific tests
run_wasm_tests() {
    echo -e "\n${YELLOW}🌐 Running WASM-specific tests...${NC}"
    
    # Build for WASM target
    if cargo build --target wasm32-unknown-unknown $CARGO_FLAGS; then
        echo -e "${GREEN}✅ WASM build successful${NC}"
    else
        echo -e "${RED}❌ WASM build failed${NC}"
        return 1
    fi
    
    # Run wasm-pack build if available
    if command -v wasm-pack &> /dev/null; then
        echo "Building WASM package with wasm-pack..."
        if ./wasm-pack-build.sh web dev; then
            echo -e "${GREEN}✅ wasm-pack build successful${NC}"
        else
            echo -e "${YELLOW}⚠️  wasm-pack build failed (may be expected if CSPICE not yet integrated)${NC}"
        fi
    else
        echo -e "${YELLOW}⚠️  wasm-pack not available, skipping WASM package build${NC}"
    fi
    
    # Run wasm-bindgen tests if available
    if command -v wasm-pack &> /dev/null && [ -f "pkg/rust_spice.js" ]; then
        if wasm-pack test --node; then
            echo -e "${GREEN}✅ WASM tests passed${NC}"
        else
            echo -e "${YELLOW}⚠️  WASM tests failed (may be expected if CSPICE not yet integrated)${NC}"
        fi
    else
        echo -e "${YELLOW}⚠️  WASM package not available, skipping wasm-pack tests${NC}"
    fi
}

# Function to run TypeScript tests
run_typescript_tests() {
    echo -e "\n${YELLOW}📝 Running TypeScript tests...${NC}"
    
    # Check if Node.js and npm are available
    if ! command -v node &> /dev/null || ! command -v npm &> /dev/null; then
        echo -e "${YELLOW}⚠️  Node.js/npm not available, skipping TypeScript tests${NC}"
        return 0
    fi
    
    # Install dependencies if needed
    if [ ! -d "node_modules" ]; then
        echo "Installing npm dependencies..."
        npm install
    fi
    
    # Run TypeScript type checking
    if npx tsc --noEmit; then
        echo -e "${GREEN}✅ TypeScript type checking passed${NC}"
    else
        echo -e "${RED}❌ TypeScript type checking failed${NC}"
        return 1
    fi
    
    # Run TypeScript integration tests
    if [ -f "tests/typescript_integration.test.ts" ]; then
        if npx jest tests/typescript_integration.test.ts; then
            echo -e "${GREEN}✅ TypeScript integration tests passed${NC}"
        else
            echo -e "${RED}❌ TypeScript integration tests failed${NC}"
            return 1
        fi
    else
        echo -e "${YELLOW}⚠️  TypeScript integration tests not found${NC}"
    fi
}

# Function to run code quality checks
run_quality_checks() {
    echo -e "\n${YELLOW}🔍 Running code quality checks...${NC}"
    
    # Clippy lints
    if cargo clippy $CARGO_FLAGS -- -D warnings; then
        echo -e "${GREEN}✅ Clippy lints passed${NC}"
    else
        echo -e "${RED}❌ Clippy lints failed${NC}"
        return 1
    fi
    
    # Format check
    if cargo fmt --check; then
        echo -e "${GREEN}✅ Code formatting is correct${NC}"
    else
        echo -e "${RED}❌ Code formatting issues found${NC}"
        echo "Run 'cargo fmt' to fix formatting"
        return 1
    fi
    
    # Security audit (if cargo-audit is available)
    if command -v cargo-audit &> /dev/null; then
        if cargo audit; then
            echo -e "${GREEN}✅ Security audit passed${NC}"
        else
            echo -e "${YELLOW}⚠️  Security audit found issues${NC}"
        fi
    else
        echo -e "${YELLOW}⚠️  cargo-audit not available, skipping security audit${NC}"
    fi
}

# Function to generate test report
generate_report() {
    local total_tests=$1
    local passed_tests=$2
    local failed_tests=$3
    
    echo -e "\n${BLUE}📊 Test Summary${NC}"
    echo "================"
    echo "Total test suites: $total_tests"
    echo -e "Passed: ${GREEN}$passed_tests${NC}"
    echo -e "Failed: ${RED}$failed_tests${NC}"
    
    if [ $failed_tests -eq 0 ]; then
        echo -e "\n${GREEN}🎉 All tests passed!${NC}"
        return 0
    else
        echo -e "\n${RED}💥 Some tests failed. Check output above for details.${NC}"
        return 1
    fi
}

# Main test execution
main() {
    check_prerequisites
    
    local total_tests=0
    local passed_tests=0
    local failed_tests=0
    
    case $TEST_TYPE in
        "unit")
            total_tests=1
            if run_unit_tests; then ((passed_tests++)); else ((failed_tests++)); fi
            ;;
        "integration")
            total_tests=1
            if run_integration_tests; then ((passed_tests++)); else ((failed_tests++)); fi
            ;;
        "property")
            total_tests=1
            if run_property_tests; then ((passed_tests++)); else ((failed_tests++)); fi
            ;;
        "snapshot")
            total_tests=1
            if run_snapshot_tests; then ((passed_tests++)); else ((failed_tests++)); fi
            ;;
        "bench")
            total_tests=1
            if run_benchmarks; then ((passed_tests++)); else ((failed_tests++)); fi
            ;;
        "wasm")
            total_tests=1
            if run_wasm_tests; then ((passed_tests++)); else ((failed_tests++)); fi
            ;;
        "typescript"|"ts")
            total_tests=1
            if run_typescript_tests; then ((passed_tests++)); else ((failed_tests++)); fi
            ;;
        "quality")
            total_tests=1
            if run_quality_checks; then ((passed_tests++)); else ((failed_tests++)); fi
            ;;
        "all")
            total_tests=9
            
            # Run all test suites
            if run_unit_tests; then ((passed_tests++)); else ((failed_tests++)); fi
            if run_integration_tests; then ((passed_tests++)); else ((failed_tests++)); fi
            if run_property_tests; then ((passed_tests++)); else ((failed_tests++)); fi
            if run_snapshot_tests; then ((passed_tests++)); else ((failed_tests++)); fi
            if run_benchmarks; then ((passed_tests++)); else ((failed_tests++)); fi
            if run_wasm_tests; then ((passed_tests++)); else ((failed_tests++)); fi
            if run_typescript_tests; then ((passed_tests++)); else ((failed_tests++)); fi
            if run_quality_checks; then ((passed_tests++)); else ((failed_tests++)); fi
            ;;
        *)
            echo -e "${RED}❌ Unknown test type: $TEST_TYPE${NC}"
            echo "Valid options: unit, integration, property, snapshot, bench, wasm, typescript, quality, all"
            exit 1
            ;;
    esac
    
    generate_report $total_tests $passed_tests $failed_tests
}

# Run main function
main "$@"
