#!/bin/bash

# Build script for RustSPICE WASM module

set -e

echo "Building RustSPICE for WASM..."

# Check if wasm-pack is installed
if ! command -v wasm-pack &> /dev/null; then
    echo "wasm-pack not found. Installing..."
    curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
fi

# Build for web target
echo "Building WASM module..."
wasm-pack build --target web --out-dir pkg

# Optimize WASM binary if wasm-opt is available
if command -v wasm-opt &> /dev/null; then
    echo "Optimizing WASM binary..."
    wasm-opt -O3 -o pkg/rust_spice_bg_optimized.wasm pkg/rust_spice_bg.wasm
    mv pkg/rust_spice_bg_optimized.wasm pkg/rust_spice_bg.wasm
fi

echo "Build complete! WASM module is in the pkg/ directory"
echo ""
echo "To use in a web project:"
echo "1. Copy the pkg/ directory to your web project"
echo "2. Import with: import init, { get_state_vector } from './pkg/rust_spice.js'"
echo "3. Initialize with: await init()"
