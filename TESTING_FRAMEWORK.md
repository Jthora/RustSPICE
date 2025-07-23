# RustSPICE Testing and Validation Framework

This document describes the comprehensive testing and validation framework for the RustSPICE project, including tools for ensuring correctness against the original NASA CSPICE toolkit.

## Overview

The testing framework consists of multiple layers of validation:

1. **Unit Tests** - Test individual components and functions
2. **Integration Tests** - Test component interactions
3. **Property-Based Tests** - Test mathematical and physical properties
4. **Snapshot Tests** - Regression testing with expected outputs
5. **Performance Benchmarks** - Performance testing and optimization
6. **CSPICE Validation** - Comparison against original CSPICE results
7. **WASM Integration Tests** - WebAssembly-specific testing

## Test Structure

```
tests/
├── integration_tests.rs     # Integration tests
├── property_tests.rs        # Property-based testing with QuickCheck/PropTest
├── snapshot_tests.rs        # Snapshot testing with insta
└── wasm_tests.rs           # WebAssembly-specific tests

benches/
└── ephemeris_benchmark.rs   # Performance benchmarking

validation/
├── test_time_conversion.c   # CSPICE reference tests
├── test_coordinates.c       # Coordinate transformation tests
├── rust_validation.rs       # Rust equivalent tests
└── Makefile                # Build validation tests

scripts/
├── run_tests.sh            # Comprehensive test runner
└── validate_against_cspice.sh  # CSPICE validation framework
```

## Running Tests

### Quick Test Run

```bash
# Run all tests
./run_tests.sh all

# Run specific test types
./run_tests.sh unit
./run_tests.sh integration
./run_tests.sh property
./run_tests.sh snapshot
./run_tests.sh bench
./run_tests.sh wasm
./run_tests.sh quality
```

### Individual Test Commands

```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test '*'

# Property-based tests
cargo test --test property_tests

# Snapshot tests
cargo test --test snapshot_tests

# Benchmarks
cargo bench

# WASM build
cargo build --target wasm32-unknown-unknown

# Code quality
cargo clippy -- -D warnings
cargo fmt --check
cargo audit  # if cargo-audit is installed
```

## CSPICE Validation

### Setup Validation Environment

```bash
# Setup validation framework (requires CSPICE toolkit)
./validate_against_cspice.sh setup

# Run complete validation
./validate_against_cspice.sh all
```

### Validation Process

1. **Build CSPICE Reference Tests** - Compile C programs using official CSPICE
2. **Generate Reference Data** - Run CSPICE tests to produce reference outputs
3. **Generate RustSPICE Data** - Run equivalent Rust implementations
4. **Compare Results** - Numerical comparison with configurable tolerance

### Validation Metrics

- **Time Conversion Accuracy** - Calendar ↔ Ephemeris Time conversions
- **Coordinate Transformations** - Cartesian ↔ Spherical/Latitudinal
- **Physical Constants** - Speed of light, astronomical unit, etc.
- **Numerical Precision** - Floating-point accuracy within tolerance

## Test Types

### 1. Unit Tests (`src/lib.rs`)

Tests individual functions and methods:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_state_vector_creation() {
        let state = StateVector::new(1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0);
        assert_eq!(state.x, 1.0);
        assert_eq!(state.light_time, 7.0);
    }
}
```

### 2. Integration Tests (`tests/integration_tests.rs`)

Tests component interactions and realistic scenarios:

```rust
#[test]
fn test_ephemeris_calculation_workflow() {
    // Test complete workflow from kernel loading to position calculation
}
```

### 3. Property-Based Tests (`tests/property_tests.rs`)

Uses QuickCheck and PropTest to test mathematical properties:

```rust
proptest! {
    #[test]
    fn test_calendar_to_et_properties(
        year in 1900i32..2100,
        month in 1i32..12,
        day in 1i32..28,
    ) {
        let et = calendar_to_et(year, month, day, 0, 0, 0.0);
        prop_assert!(et.is_finite());
    }
}
```

Properties tested:
- **Monotonicity** - Later dates have larger ephemeris times
- **Conservation** - Physical laws are preserved
- **Symmetry** - Distance calculations are symmetric
- **Range Validation** - Results are within reasonable bounds

### 4. Snapshot Tests (`tests/snapshot_tests.rs`)

Uses `insta` for regression testing with expected outputs:

```rust
#[test]
fn test_planetary_positions_snapshot() {
    let earth_state = StateVector::new(149597870.7, 0.0, 0.0, 0.0, 29.78, 0.0, 0.0);
    assert_debug_snapshot!(earth_state);
}
```

Snapshots capture:
- **State Vector Outputs** - Position and velocity data
- **Time Conversions** - Known epoch conversions
- **Error Messages** - Consistent error formatting
- **Mission Scenarios** - Typical space mission calculations

### 5. Performance Benchmarks (`benches/ephemeris_benchmark.rs`)

Uses Criterion for performance testing:

```rust
fn bench_spkezr_call(c: &mut Criterion) {
    c.bench_function("spkezr Earth-Moon", |b| {
        b.iter(|| {
            // Benchmark ephemeris calculations
        })
    });
}
```

Benchmarks include:
- **Position Calculations** - SPKEZR equivalent functions
- **Time Conversions** - Calendar/Julian date processing
- **Coordinate Transformations** - Reference frame conversions
- **Memory Allocation** - WASM heap usage patterns

### 6. WASM Integration Tests (`tests/wasm_tests.rs`)

WebAssembly-specific testing:

```rust
#[wasm_bindgen_test]
fn test_wasm_kernel_loading() {
    // Test loading kernels in WASM environment
}
```

Tests:
- **ArrayBuffer Loading** - Binary kernel data loading
- **Memory Management** - WASM heap handling
- **JavaScript Interop** - TypeScript interface correctness
- **Error Propagation** - Cross-boundary error handling

## Validation Against CSPICE

### Reference Test Generation

The validation framework creates C programs that use official CSPICE functions:

```c
#include "SpiceUsr.h"

int main() {
    double et;
    str2et_c("2000-01-01T12:00:00", &et);
    printf("J2000_ET: %.15f\n", et);
    return 0;
}
```

### Comparison Methodology

1. **Numerical Tolerance** - Configurable floating-point tolerance (default: 1e-10)
2. **Vector Comparisons** - Element-wise comparison for position/velocity vectors
3. **Statistical Analysis** - Mean absolute error and maximum deviation
4. **Regression Detection** - Automated detection of accuracy degradation

### Validation Scenarios

- **Historical Missions** - Apollo 11, Galileo, Cassini, New Horizons
- **Planetary Positions** - Major planets at various epochs
- **Time Systems** - UTC, TDB, ET conversions
- **Coordinate Frames** - Ecliptic, equatorial, planetary frames

## Continuous Integration

The testing framework supports CI/CD integration:

```yaml
# Example GitHub Actions workflow
- name: Run comprehensive tests
  run: ./run_tests.sh all

- name: Validate against CSPICE
  run: ./validate_against_cspice.sh all

- name: Upload test artifacts
  uses: actions/upload-artifact@v3
  with:
    name: test-results
    path: |
      target/criterion/
      validation/
```

## Quality Metrics

### Code Coverage

```bash
# Install cargo-tarpaulin for coverage
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage/
```

### Performance Tracking

- **Benchmark History** - Track performance over time
- **Memory Usage** - Monitor WASM heap usage
- **Bundle Size** - Track WASM binary size
- **Load Time** - Measure initialization performance

### Accuracy Metrics

- **Maximum Absolute Error** - Worst-case accuracy
- **Root Mean Square Error** - Overall accuracy
- **Relative Error** - Percentage-based accuracy
- **Significant Digits** - Precision maintenance

## Testing Tools Summary

| Tool | Purpose | Integration |
|------|---------|-------------|
| **Rust built-in** | Unit and integration tests | `cargo test` |
| **PropTest** | Property-based testing | Automatic generation |
| **QuickCheck** | Property testing | Simple property validation |
| **Insta** | Snapshot testing | Regression detection |
| **Criterion** | Performance benchmarking | Statistical analysis |
| **wasm-bindgen-test** | WASM testing | Browser/Node.js testing |
| **Custom C programs** | CSPICE validation | Reference comparison |

## Best Practices

### Test Organization

1. **Separate Concerns** - Different test types in different files
2. **Clear Naming** - Descriptive test names indicating what's being tested
3. **Documentation** - Comments explaining complex test scenarios
4. **Data Management** - Separate test data from test logic

### Property Selection

1. **Mathematical Properties** - Monotonicity, conservation, symmetry
2. **Physical Laws** - Energy conservation, momentum preservation
3. **Boundary Conditions** - Edge cases and limits
4. **Invariants** - Properties that should always hold

### Performance Testing

1. **Realistic Scenarios** - Test with mission-relevant data
2. **Multiple Scales** - Test from microsecond to century timescales
3. **Memory Patterns** - Test typical usage patterns
4. **Regression Prevention** - Alert on performance degradation

## Troubleshooting

### Common Issues

1. **CSPICE Build Failures** - Ensure FORTRAN compiler availability
2. **WASM Target Missing** - Install with `rustup target add wasm32-unknown-unknown`
3. **Numerical Precision** - Adjust tolerance for platform differences
4. **Memory Limits** - Increase WASM memory limits for large kernels

### Debug Techniques

1. **Verbose Output** - Use `./run_tests.sh all -v` for detailed output
2. **Isolated Testing** - Run individual test suites to isolate issues
3. **Tolerance Adjustment** - Temporary tolerance increase for debugging
4. **Reference Comparison** - Compare intermediate values with CSPICE

## Future Enhancements

1. **Fuzz Testing** - Add comprehensive fuzz testing
2. **GPU Testing** - Test GPU-accelerated computations
3. **Distributed Testing** - Parallel testing across multiple environments
4. **Visual Validation** - Orbital plot comparisons
5. **Real-time Testing** - Continuous validation during development

This comprehensive testing framework ensures that RustSPICE maintains accuracy and performance equivalent to the original NASA CSPICE toolkit while providing the benefits of Rust's safety and WebAssembly's portability.
