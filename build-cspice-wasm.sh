#!/bin/bash

# Build CSPICE to WASM using Emscripten
# This script compiles the essential CSPICE functions to WebAssembly

set -e

echo "=== Building CSPICE for WASM ==="

# Check if Emscripten is available
if ! command -v emcc &> /dev/null; then
    echo "ERROR: Emscripten not found. Please install Emscripten SDK:"
    echo "  git clone https://github.com/emscripten-core/emsdk.git"
    echo "  cd emsdk && ./emsdk install latest && ./emsdk activate latest"
    echo "  source ./emsdk_env.sh"
    exit 1
fi

# Create build directory
mkdir -p cspice-wasm
cd cspice-wasm

# CSPICE source directory
CSPICE_SRC="../cspice/cspice/src/cspice"
CSPICE_INCLUDE="../cspice/cspice/include"

# Core CSPICE C files to compile (avoiding FORTRAN for now)
CSPICE_FILES=(
    # Core state vector functions
    "spkezr_c.c"
    "spkezp_c.c" 
    "spkpos_c.c"
    
    # Kernel loading
    "furnsh_c.c"
    "unload_c.c"
    "kclear_c.c"
    
    # Error handling
    "failed_c.c"
    "getmsg_c.c"
    "reset_c.c"
    "sigerr_c.c"
    "setmsg_c.c"
    
    # Time conversions
    "str2et_c.c"
    "et2utc_c.c"
    "utc2et_c.c"
    
    # Coordinate transformations
    "pxform_c.c"
    "sxform_c.c"
    
    # Frame functions
    "namfrm_c.c"
    "frmnam_c.c"
    
    # Essential utilities
    "chkin_c.c"
    "chkout_c.c"
    "erract_c.c"
    "errdev_c.c"
    "exists_c.c"
)

echo "Collecting CSPICE source files..."

# Copy essential CSPICE files
for file in "${CSPICE_FILES[@]}"; do
    if [[ -f "$CSPICE_SRC/$file" ]]; then
        cp "$CSPICE_SRC/$file" .
        echo "  ✓ $file"
    else
        echo "  ✗ $file (not found)"
    fi
done

# Copy headers
cp -r "$CSPICE_INCLUDE"/* .

echo ""
echo "Compiling CSPICE to WASM..."

# Emscripten compile flags
EMCC_FLAGS=(
    -O2                                    # Optimize for size/speed
    -s WASM=1                             # Generate WASM
    -s ALLOW_MEMORY_GROWTH=1              # Allow memory to grow
    -s EXPORTED_RUNTIME_METHODS='["ccall","cwrap"]'  # Export call helpers
    -s MODULARIZE=1                       # Create Module function
    -s EXPORT_NAME="'CSPICEModule'"       # Module name
    -I.                                   # Include current directory
    
    # Export essential functions
    -s EXPORTED_FUNCTIONS='[
        "_spkezr_c",
        "_spkezp_c", 
        "_spkpos_c",
        "_furnsh_c",
        "_unload_c", 
        "_kclear_c",
        "_failed_c",
        "_getmsg_c",
        "_reset_c",
        "_sigerr_c",
        "_setmsg_c",
        "_str2et_c",
        "_et2utc_c",
        "_utc2et_c",
        "_pxform_c",
        "_sxform_c",
        "_namfrm_c",
        "_frmnam_c",
        "_chkin_c",
        "_chkout_c",
        "_erract_c",
        "_errdev_c",
        "_exists_c"
    ]'
)

# Try to compile - this will likely fail due to FORTRAN dependencies
echo "Attempting WASM compilation..."
if emcc "${EMCC_FLAGS[@]}" *.c -o cspice.js 2>build_errors.log; then
    echo "✓ SUCCESS: CSPICE compiled to WASM!"
    echo "  Generated: cspice.wasm, cspice.js"
    ls -lh cspice.*
else
    echo "✗ COMPILATION FAILED (expected due to FORTRAN dependencies)"
    echo ""
    echo "Build errors (first 20 lines):"
    head -20 build_errors.log
    echo ""
    echo "This is expected - CSPICE C wrappers depend on FORTRAN functions."
    echo "Next steps:"
    echo "1. Install LLVM Flang with WASM patches"
    echo "2. Compile FORTRAN source to WASM objects"
    echo "3. Link C and FORTRAN objects together"
    echo ""
    echo "For now, this gives us the structure and identifies dependencies."
fi

echo ""
echo "=== Analysis ==="
echo "C files found: $(ls -1 *.c | wc -l)"
echo "Header files found: $(ls -1 *.h | wc -l)"
echo "Build directory: $(pwd)"

# Show what FORTRAN functions are needed
echo ""
echo "FORTRAN dependencies found in build errors:"
if [[ -f build_errors.log ]]; then
    grep -o "undefined symbol: [a-zA-Z_][a-zA-Z0-9_]*" build_errors.log | \
    grep -E "_$" | sort | uniq | head -10
fi

echo ""
echo "Next: Implement stub functions or compile FORTRAN sources"
