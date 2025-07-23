#!/bin/bash

# CSPICE Function Analysis Tool
# Analyzes the CSPICE codebase to understand dependencies and complexity

set -e

CSPICE_DIR="./cspice/cspice/src/cspice"
OUTPUT_DIR="./analysis"

echo "🔍 CSPICE Codebase Analysis"
echo "=============================="

# Create output directory
mkdir -p "$OUTPUT_DIR"

echo "📊 Analyzing CSPICE codebase structure..."

# Count different types of files
C_WRAPPERS=$(ls -1 "$CSPICE_DIR"/*_c.c 2>/dev/null | wc -l)
CORE_FUNCTIONS=$(ls -1 "$CSPICE_DIR"/*.c | grep -v "_c.c" | wc -l)
HEADER_FILES=$(ls -1 "$CSPICE_DIR"/*.h 2>/dev/null | wc -l)

echo "📈 File Statistics:"
echo "   C Wrapper Functions (*_c.c): $C_WRAPPERS"
echo "   Core Functions (*.c): $CORE_FUNCTIONS"  
echo "   Header Files (*.h): $HEADER_FILES"
echo "   Total C files: $((C_WRAPPERS + CORE_FUNCTIONS))"

# Analyze function categories
echo ""
echo "🗂️  Function Categories:"

# SPK (Ephemeris) functions
SPK_COUNT=$(ls -1 "$CSPICE_DIR"/spk*.c 2>/dev/null | wc -l)
echo "   SPK (Ephemeris): $SPK_COUNT functions"

# PCK (Planetary Constants) functions  
PCK_COUNT=$(ls -1 "$CSPICE_DIR"/pck*.c 2>/dev/null | wc -l)
echo "   PCK (Planetary Constants): $PCK_COUNT functions"

# CK (Pointing) functions
CK_COUNT=$(ls -1 "$CSPICE_DIR"/ck*.c 2>/dev/null | wc -l)
echo "   CK (Pointing): $CK_COUNT functions"

# Time functions
TIME_COUNT=$(ls -1 "$CSPICE_DIR"/*time*.c "$CSPICE_DIR"/*et*.c "$CSPICE_DIR"/*utc*.c 2>/dev/null | wc -l)
echo "   Time Systems: $TIME_COUNT functions"

# Vector/Matrix math
MATH_COUNT=$(ls -1 "$CSPICE_DIR"/v*.c "$CSPICE_DIR"/m*.c 2>/dev/null | wc -l)
echo "   Vector/Matrix Math: $MATH_COUNT functions"

# Coordinate transformations
COORD_COUNT=$(ls -1 "$CSPICE_DIR"/*rec*.c "$CSPICE_DIR"/rec*.c 2>/dev/null | wc -l)
echo "   Coordinate Transforms: $COORD_COUNT functions"

# DAF/DAS file system
FILE_COUNT=$(ls -1 "$CSPICE_DIR"/daf*.c "$CSPICE_DIR"/das*.c 2>/dev/null | wc -l)
echo "   File I/O (DAF/DAS): $FILE_COUNT functions"

# Error handling
ERROR_COUNT=$(ls -1 "$CSPICE_DIR"/*err*.c "$CSPICE_DIR"/chk*.c "$CSPICE_DIR"/sig*.c 2>/dev/null | wc -l)
echo "   Error Handling: $ERROR_COUNT functions"

echo ""
echo "🎯 Priority Function Analysis:"

# Extract the most important user-facing functions
cat > "$OUTPUT_DIR/priority_functions.txt" << 'EOF'
# Priority 1: Essential Ephemeris Functions
spkezr_c.c          # Get state vector (position + velocity)
spkpos_c.c          # Get position vector
spkgeo_c.c          # Get geometric state

# Priority 2: Time System Functions  
str2et_c.c          # String to ephemeris time
et2utc_c.c          # Ephemeris time to UTC
utc2et_c.c          # UTC to ephemeris time
timout_c.c          # Time formatting

# Priority 3: Coordinate Transformations
pxform_c.c          # Position transformation matrix
sxform_c.c          # State transformation matrix
reclat_c.c          # Rectangular to latitudinal
latrec_c.c          # Latitudinal to rectangular
recsph_c.c          # Rectangular to spherical
sphrec_c.c          # Spherical to rectangular

# Priority 4: Kernel Management
furnsh_c.c          # Load kernels
unload_c.c          # Unload kernels
kclear_c.c          # Clear all kernels

# Priority 5: Planetary Constants
bodvrd_c.c          # Get body variable real data
bodn2c_c.c          # Body name to code
bodc2n_c.c          # Body code to name

# Priority 6: Mathematical Operations
vdot_c.c            # Vector dot product
vcrss_c.c           # Vector cross product
vnorm_c.c           # Vector norm
vhat_c.c            # Unit vector
mxm_c.c             # Matrix multiply
mxv_c.c             # Matrix times vector
EOF

echo "✅ Created priority function list: $OUTPUT_DIR/priority_functions.txt"

# Analyze function dependencies
echo ""
echo "🔗 Analyzing Function Dependencies..."

# Create dependency analysis
cat > "$OUTPUT_DIR/analyze_dependencies.py" << 'EOF'
#!/usr/bin/env python3
"""
Analyze CSPICE function dependencies by parsing C files
"""

import os
import re
import sys
from collections import defaultdict, deque

def extract_function_calls(filepath):
    """Extract function calls from a C file"""
    calls = set()
    try:
        with open(filepath, 'r', encoding='utf-8', errors='ignore') as f:
            content = f.read()
            
        # Remove comments and strings to avoid false positives
        content = re.sub(r'/\*.*?\*/', '', content, flags=re.DOTALL)
        content = re.sub(r'//.*?\n', '\n', content)
        content = re.sub(r'".*?"', '""', content)
        
        # Find function calls (word followed by opening parenthesis)
        pattern = r'\b([a-zA-Z_][a-zA-Z0-9_]*)\s*\('
        matches = re.findall(pattern, content)
        
        # Filter out common C keywords and likely non-function calls
        keywords = {'if', 'for', 'while', 'switch', 'sizeof', 'return', 'typedef'}
        calls = {match for match in matches if match not in keywords and len(match) > 2}
        
    except Exception as e:
        print(f"Error reading {filepath}: {e}")
        
    return calls

def main():
    cspice_dir = "./cspice/cspice/src/cspice"
    if not os.path.exists(cspice_dir):
        print(f"CSPICE directory not found: {cspice_dir}")
        return
        
    # Get all C files
    c_files = [f for f in os.listdir(cspice_dir) if f.endswith('.c')]
    
    dependencies = {}
    
    print(f"Analyzing {len(c_files)} C files for dependencies...")
    
    for i, filename in enumerate(c_files):
        if i % 100 == 0:
            print(f"  Progress: {i}/{len(c_files)}")
            
        filepath = os.path.join(cspice_dir, filename)
        calls = extract_function_calls(filepath)
        dependencies[filename] = calls
    
    # Find most commonly called functions
    call_counts = defaultdict(int)
    for filename, calls in dependencies.items():
        for call in calls:
            call_counts[call] += 1
    
    # Write results
    with open("./analysis/function_dependencies.txt", "w") as f:
        f.write("CSPICE Function Dependency Analysis\n")
        f.write("===================================\n\n")
        
        f.write("Most Called Functions (likely core utilities):\n")
        for func, count in sorted(call_counts.items(), key=lambda x: x[1], reverse=True)[:20]:
            f.write(f"  {func}: called by {count} files\n")
        
        f.write("\n\nDetailed Dependencies:\n")
        for filename in sorted(dependencies.keys()):
            calls = dependencies[filename]
            if calls:
                f.write(f"\n{filename}:\n")
                for call in sorted(calls):
                    f.write(f"  -> {call}\n")
    
    print("✅ Dependency analysis complete: ./analysis/function_dependencies.txt")

if __name__ == "__main__":
    main()
EOF

python3 "$OUTPUT_DIR/analyze_dependencies.py"

# Analyze code complexity
echo ""
echo "📏 Analyzing Code Complexity..."

find "$CSPICE_DIR" -name "*.c" -exec wc -l {} + | sort -n > "$OUTPUT_DIR/file_sizes.txt"

echo "📊 Largest files (lines of code):"
tail -20 "$OUTPUT_DIR/file_sizes.txt" | head -19

# Calculate total lines of code
TOTAL_LINES=$(awk '{sum += $1} END {print sum}' "$OUTPUT_DIR/file_sizes.txt")
echo ""
echo "📈 Total Lines of Code: $TOTAL_LINES"

# Estimate conversion effort
echo ""
echo "⏱️  Conversion Effort Estimation:"
echo "   Based on $TOTAL_LINES lines of C code:"
echo "   - Conservative estimate: $((TOTAL_LINES / 100)) person-hours"
echo "   - Aggressive estimate: $((TOTAL_LINES / 200)) person-hours"
echo "   - With testing/validation: $((TOTAL_LINES / 50)) person-hours"

# Create conversion tracking template
cat > "$OUTPUT_DIR/conversion_progress.md" << 'EOF'
# RustSPICE Conversion Progress

## Overview
- **Total C Files**: TOTAL_FILES
- **Total Lines**: TOTAL_LINES  
- **Estimated Effort**: EFFORT_HOURS hours

## Conversion Status

### Phase 1: Foundation (Completed: 0/X)
- [ ] Error handling system
- [ ] Core data types  
- [ ] Mathematical operations
- [ ] Memory management

### Phase 2: Time System (Completed: 0/Y)
- [ ] str2et_c.c → str_to_et()
- [ ] et2utc_c.c → et_to_utc()
- [ ] utc2et_c.c → utc_to_et()
- [ ] timout_c.c → time_output()

### Phase 3: Coordinates (Completed: 0/Z)
- [ ] pxform_c.c → position_transform()
- [ ] sxform_c.c → state_transform()
- [ ] Coordinate conversion functions

### Phase 4: File I/O (Completed: 0/A)
- [ ] Virtual file system
- [ ] DAF/DAS implementation
- [ ] Kernel loading system

### Phase 5: Ephemeris (Completed: 0/B)
- [ ] spkezr_c.c → ephemeris_state()
- [ ] spkpos_c.c → ephemeris_position()
- [ ] SPK interpolation algorithms

## Detailed Progress

### High Priority Functions
| Function | Status | Lines | Complexity | Notes |
|----------|--------|-------|------------|-------|
| spkezr_c.c | ❌ Not Started | XXX | High | Core ephemeris |
| spkpos_c.c | ❌ Not Started | XXX | High | Position only |
| str2et_c.c | ❌ Not Started | XXX | Medium | Time parsing |
| et2utc_c.c | ❌ Not Started | XXX | Medium | Time formatting |

### Medium Priority Functions
| Function | Status | Lines | Complexity | Notes |
|----------|--------|-------|------------|-------|
| (TBD) | | | | |

### Low Priority Functions  
| Function | Status | Lines | Complexity | Notes |
|----------|--------|-------|------------|-------|
| (TBD) | | | | |

## Testing Status
- [ ] Unit test framework
- [ ] CSPICE validation suite
- [ ] Performance benchmarks
- [ ] WASM integration tests

## Build System Status
- [ ] Rust library structure
- [ ] WASM build pipeline
- [ ] TypeScript bindings
- [ ] NPM package setup
EOF

# Replace placeholders in the template
sed -i "s/TOTAL_FILES/$((C_WRAPPERS + CORE_FUNCTIONS))/g" "$OUTPUT_DIR/conversion_progress.md"
sed -i "s/TOTAL_LINES/$TOTAL_LINES/g" "$OUTPUT_DIR/conversion_progress.md"
sed -i "s/EFFORT_HOURS/$((TOTAL_LINES / 100))/g" "$OUTPUT_DIR/conversion_progress.md"

echo ""
echo "✅ Analysis Complete!"
echo ""
echo "📋 Generated Files:"
echo "   - $OUTPUT_DIR/priority_functions.txt"
echo "   - $OUTPUT_DIR/function_dependencies.txt"
echo "   - $OUTPUT_DIR/file_sizes.txt" 
echo "   - $OUTPUT_DIR/conversion_progress.md"
echo ""
echo "🚀 Next Steps:"
echo "   1. Review priority functions and dependencies"
echo "   2. Start with Phase 1: Foundation implementation"
echo "   3. Set up Rust project structure for conversion"
echo "   4. Create validation framework against CSPICE"
