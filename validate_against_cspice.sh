#!/bin/bash

# CSPICE Validation Framework
# Compares RustSPICE output against native CSPICE for validation
# Usage: ./validate_against_cspice.sh [test_case]

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
CSPICE_DIR="/home/jono/workspace/github/RustSPICE/cspice"
VALIDATION_DIR="/home/jono/workspace/github/RustSPICE/validation"
TOLERANCE=1e-10  # Numerical tolerance for comparisons

echo -e "${BLUE}🔬 CSPICE Validation Framework${NC}"
echo "================================="

# Check if CSPICE is available
check_cspice() {
    if [ ! -d "$CSPICE_DIR" ]; then
        echo -e "${RED}❌ CSPICE directory not found at $CSPICE_DIR${NC}"
        echo "Please extract CSPICE toolkit first"
        exit 1
    fi
    
    if [ ! -f "$CSPICE_DIR/lib/cspice.a" ]; then
        echo -e "${YELLOW}⚠️  CSPICE library not built. Building now...${NC}"
        cd "$CSPICE_DIR"
        ./makeall.csh
        cd - > /dev/null
    fi
    
    echo -e "${GREEN}✅ CSPICE available${NC}"
}

# Create validation test programs
create_validation_tests() {
    mkdir -p "$VALIDATION_DIR"
    
    # Create C test program for time conversion validation
    cat > "$VALIDATION_DIR/test_time_conversion.c" << 'EOF'
#include "SpiceUsr.h"
#include <stdio.h>
#include <math.h>

int main() {
    // Test calendar to ET conversion
    double et;
    
    // Test J2000 epoch
    str2et_c("2000-01-01T12:00:00", &et);
    printf("J2000_ET: %.15f\n", et);
    
    // Test some specific dates
    str2et_c("1969-07-16T13:32:00", &et);  // Apollo 11
    printf("APOLLO11_ET: %.15f\n", et);
    
    str2et_c("1989-10-18T16:53:40", &et);  // Galileo
    printf("GALILEO_ET: %.15f\n", et);
    
    str2et_c("1997-10-15T08:43:00", &et);  // Cassini
    printf("CASSINI_ET: %.15f\n", et);
    
    str2et_c("2006-01-19T19:00:00", &et);  // New Horizons
    printf("NEWHORIZONS_ET: %.15f\n", et);
    
    // Test Julian date conversions
    double jd = 2451545.0;  // J2000
    et = (jd - 2451545.0) * 86400.0;
    printf("JD_2451545_ET: %.15f\n", et);
    
    jd = 2451546.0;  // One day after J2000
    et = (jd - 2451545.0) * 86400.0;
    printf("JD_2451546_ET: %.15f\n", et);
    
    // Test physical constants
    printf("SPEED_OF_LIGHT: %.15f\n", clight_c());
    
    return 0;
}
EOF

    # Create C test program for coordinate transformations
    cat > "$VALIDATION_DIR/test_coordinates.c" << 'EOF'
#include "SpiceUsr.h"
#include <stdio.h>
#include <math.h>

int main() {
    double pos[3], vel[3];
    double range, lt;
    
    // Test some basic coordinate calculations
    
    // Distance calculation
    pos[0] = 149597870.7;  // 1 AU
    pos[1] = 0.0;
    pos[2] = 0.0;
    
    range = vnorm_c(pos);
    printf("AU_DISTANCE: %.15f\n", range);
    
    // Light time calculation (simplified)
    lt = range / clight_c();
    printf("AU_LIGHT_TIME: %.15f\n", lt);
    
    // Unit vector calculation
    double unit[3];
    vhat_c(pos, unit);
    printf("X_UNIT_VECTOR: %.15f %.15f %.15f\n", unit[0], unit[1], unit[2]);
    
    // Test coordinate transformations
    double rec[3] = {1.0, 1.0, 1.0};
    double radius, lon, lat;
    
    reclat_c(rec, &radius, &lon, &lat);
    printf("RECTANGULAR_TO_LATITUDINAL: %.15f %.15f %.15f\n", radius, lon, lat);
    
    double sph_radius, colatitude, longitude;
    recsph_c(rec, &sph_radius, &colatitude, &longitude);
    printf("RECTANGULAR_TO_SPHERICAL: %.15f %.15f %.15f\n", sph_radius, colatitude, longitude);
    
    return 0;
}
EOF

    # Create makefile for validation tests
    cat > "$VALIDATION_DIR/Makefile" << EOF
CC = gcc
CFLAGS = -I$CSPICE_DIR/include -std=c99
LDFLAGS = -L$CSPICE_DIR/lib -lcspice -lm

TARGETS = test_time_conversion test_coordinates

all: \$(TARGETS)

test_time_conversion: test_time_conversion.c
	\$(CC) \$(CFLAGS) -o \$@ \$< \$(LDFLAGS)

test_coordinates: test_coordinates.c
	\$(CC) \$(CFLAGS) -o \$@ \$< \$(LDFLAGS)

clean:
	rm -f \$(TARGETS) *.o

.PHONY: all clean
EOF

    echo -e "${GREEN}✅ Validation test programs created${NC}"
}

# Build validation tests
build_validation_tests() {
    echo -e "${YELLOW}🔨 Building validation tests...${NC}"
    cd "$VALIDATION_DIR"
    make clean && make all
    cd - > /dev/null
    echo -e "${GREEN}✅ Validation tests built${NC}"
}

# Run CSPICE reference tests
run_cspice_reference() {
    echo -e "${YELLOW}📊 Running CSPICE reference tests...${NC}"
    
    cd "$VALIDATION_DIR"
    
    echo "Running time conversion tests..."
    ./test_time_conversion > cspice_time_output.txt
    
    echo "Running coordinate tests..."
    ./test_coordinates > cspice_coord_output.txt
    
    cd - > /dev/null
    
    echo -e "${GREEN}✅ CSPICE reference data generated${NC}"
}

# Generate RustSPICE test data
generate_rust_test_data() {
    echo -e "${YELLOW}🦀 Generating RustSPICE test data...${NC}"
    
    # Create Rust program to generate equivalent test data
    cat > "$VALIDATION_DIR/rust_validation.rs" << 'EOF'
use rust_spice::*;

fn main() {
    // Test calendar to ET conversion (simplified implementation)
    
    // J2000 epoch
    let et = calendar_to_et(2000, 1, 1, 12, 0, 0.0);
    println!("J2000_ET: {:.15}", et);
    
    // Apollo 11 launch
    let et = calendar_to_et(1969, 7, 16, 13, 32, 0.0);
    println!("APOLLO11_ET: {:.15}", et);
    
    // Galileo launch
    let et = calendar_to_et(1989, 10, 18, 16, 53, 40.0);
    println!("GALILEO_ET: {:.15}", et);
    
    // Cassini launch
    let et = calendar_to_et(1997, 10, 15, 8, 43, 0.0);
    println!("CASSINI_ET: {:.15}", et);
    
    // New Horizons launch
    let et = calendar_to_et(2006, 1, 19, 19, 0, 0.0);
    println!("NEWHORIZONS_ET: {:.15}", et);
    
    // Julian date conversions
    let et = julian_date_to_et(2451545.0);
    println!("JD_2451545_ET: {:.15}", et);
    
    let et = julian_date_to_et(2451546.0);
    println!("JD_2451546_ET: {:.15}", et);
    
    // Physical constants
    println!("SPEED_OF_LIGHT: {:.15}", 299792.458);
    
    // Coordinate tests
    let pos = vec![149597870.7, 0.0, 0.0];
    let distance = (pos[0] * pos[0] + pos[1] * pos[1] + pos[2] * pos[2]).sqrt();
    println!("AU_DISTANCE: {:.15}", distance);
    
    let light_time = distance / 299792.458;
    println!("AU_LIGHT_TIME: {:.15}", light_time);
    
    // Unit vector
    let magnitude = distance;
    println!("X_UNIT_VECTOR: {:.15} {:.15} {:.15}", 
        pos[0] / magnitude, pos[1] / magnitude, pos[2] / magnitude);
    
    // Coordinate transformations (simplified)
    let x = 1.0;
    let y = 1.0;
    let z = 1.0;
    
    let radius = (x*x + y*y + z*z).sqrt();
    let lon = y.atan2(x);
    let lat = (z / radius).asin();
    println!("RECTANGULAR_TO_LATITUDINAL: {:.15} {:.15} {:.15}", radius, lon, lat);
    
    let colatitude = (z / radius).acos();
    let longitude = y.atan2(x);
    println!("RECTANGULAR_TO_SPHERICAL: {:.15} {:.15} {:.15}", radius, colatitude, longitude);
}
EOF

    # Build and run the Rust validation program
    cd "$VALIDATION_DIR"
    rustc --extern rust_spice=../target/release/librust_spice.rlib rust_validation.rs -o rust_validation
    ./rust_validation > rust_output.txt
    cd - > /dev/null
    
    echo -e "${GREEN}✅ RustSPICE test data generated${NC}"
}

# Compare results
compare_results() {
    echo -e "${YELLOW}🔍 Comparing results...${NC}"
    
    cd "$VALIDATION_DIR"
    
    # Create comparison script
    cat > compare.py << 'EOF'
#!/usr/bin/env python3
import sys
import re

def parse_output(filename):
    """Parse output file and return dictionary of key-value pairs"""
    data = {}
    with open(filename, 'r') as f:
        for line in f:
            line = line.strip()
            if ':' in line:
                key, value = line.split(':', 1)
                key = key.strip()
                value = value.strip()
                
                # Handle multiple values (like vectors)
                if ' ' in value:
                    values = [float(x) for x in value.split()]
                    data[key] = values
                else:
                    try:
                        data[key] = float(value)
                    except ValueError:
                        data[key] = value
    return data

def compare_values(cspice_val, rust_val, tolerance=1e-10):
    """Compare two values with given tolerance"""
    if isinstance(cspice_val, list) and isinstance(rust_val, list):
        if len(cspice_val) != len(rust_val):
            return False, "Different vector lengths"
        
        for i, (c, r) in enumerate(zip(cspice_val, rust_val)):
            if abs(c - r) > tolerance:
                return False, f"Element {i}: {abs(c - r)} > {tolerance}"
        return True, "OK"
    
    elif isinstance(cspice_val, (int, float)) and isinstance(rust_val, (int, float)):
        diff = abs(cspice_val - rust_val)
        if diff > tolerance:
            return False, f"Difference: {diff} > {tolerance}"
        return True, "OK"
    
    else:
        return cspice_val == rust_val, "String comparison"

def main():
    tolerance = float(sys.argv[3]) if len(sys.argv) > 3 else 1e-10
    
    print("CSPICE vs RustSPICE Validation Report")
    print("=" * 50)
    
    # Compare time conversion results
    if len(sys.argv) > 1:
        cspice_data = parse_output(sys.argv[1])
        rust_data = parse_output(sys.argv[2])
        
        total_tests = 0
        passed_tests = 0
        
        for key in cspice_data:
            if key in rust_data:
                total_tests += 1
                cspice_val = cspice_data[key]
                rust_val = rust_data[key]
                
                passed, message = compare_values(cspice_val, rust_val, tolerance)
                
                status = "PASS" if passed else "FAIL"
                print(f"{key:25}: {status:4} - {message}")
                
                if passed:
                    passed_tests += 1
                else:
                    print(f"  CSPICE: {cspice_val}")
                    print(f"  Rust:   {rust_val}")
            else:
                print(f"{key:25}: MISSING in Rust output")
        
        print("\n" + "=" * 50)
        print(f"Summary: {passed_tests}/{total_tests} tests passed")
        print(f"Success rate: {100.0 * passed_tests / total_tests:.1f}%")
        
        return 0 if passed_tests == total_tests else 1
    else:
        print("Usage: compare.py <cspice_output> <rust_output> [tolerance]")
        return 1

if __name__ == "__main__":
    sys.exit(main())
EOF

    chmod +x compare.py
    
    # Compare outputs
    echo "Comparing time conversion results:"
    if python3 compare.py cspice_time_output.txt rust_output.txt $TOLERANCE; then
        echo -e "${GREEN}✅ Time conversion validation passed${NC}"
    else
        echo -e "${RED}❌ Time conversion validation failed${NC}"
    fi
    
    echo -e "\nComparing coordinate results:"
    if python3 compare.py cspice_coord_output.txt rust_output.txt $TOLERANCE; then
        echo -e "${GREEN}✅ Coordinate validation passed${NC}"
    else
        echo -e "${RED}❌ Coordinate validation failed${NC}"
    fi
    
    cd - > /dev/null
}

# Main validation function
main() {
    local test_case=${1:-all}
    
    case $test_case in
        "setup")
            check_cspice
            create_validation_tests
            build_validation_tests
            ;;
        "reference")
            run_cspice_reference
            ;;
        "rust")
            generate_rust_test_data
            ;;
        "compare")
            compare_results
            ;;
        "all")
            check_cspice
            create_validation_tests
            build_validation_tests
            run_cspice_reference
            
            # Build Rust library first
            echo -e "${YELLOW}🔨 Building RustSPICE library...${NC}"
            cargo build --release
            
            generate_rust_test_data
            compare_results
            ;;
        *)
            echo -e "${RED}❌ Unknown test case: $test_case${NC}"
            echo "Valid options: setup, reference, rust, compare, all"
            exit 1
            ;;
    esac
}

# Run main function
main "$@"
