#!/bin/bash

# RustSPICE Repository Analysis Script

echo "🔬 RustSPICE Repository Analysis"
echo "================================"

# Core project files
echo "📁 Core Project Files:"
echo "  Source code:"
find src/ -name "*.rs" 2>/dev/null | wc -l | xargs echo "    Rust files:"
find tests/ -name "*.rs" 2>/dev/null | wc -l | xargs echo "    Test files:"
find benches/ -name "*.rs" 2>/dev/null | wc -l | xargs echo "    Benchmark files:"

echo "  Configuration:"
ls -1 *.toml *.json *.md 2>/dev/null | wc -l | xargs echo "    Config/doc files:"

# CSPICE analysis
echo ""
echo "📚 CSPICE Reference Code:"
if [[ -d "cspice/cspice" ]]; then
    echo "  Source files for conversion reference:"
    find cspice/cspice/src -name "*.c" 2>/dev/null | wc -l | xargs echo "    C source files:"
    find cspice/cspice/include -name "*.h" 2>/dev/null | wc -l | xargs echo "    Header files:"
    
    echo "  Documentation:"
    find cspice/cspice/doc -name "*.req" 2>/dev/null | wc -l | xargs echo "    Requirement docs:"
    find cspice/cspice/doc -name "*.ug" 2>/dev/null | wc -l | xargs echo "    User guides:"
    
    echo "  Test/example data:"
    find cspice/cspice/data -type f 2>/dev/null | wc -l | xargs echo "    Data files:"
    
    echo "  Binaries (should be ignored):"
    find cspice/cspice/lib -name "*.a" 2>/dev/null | wc -l | xargs echo "    Library files:"
    find cspice/cspice/exe -type f 2>/dev/null | wc -l | xargs echo "    Executables:"
fi

# Repository status
echo ""
echo "📊 Repository Status:"
echo "  Git tracking:"
git ls-files 2>/dev/null | wc -l | xargs echo "    Tracked files:"
git status --porcelain 2>/dev/null | grep "^??" | wc -l | xargs echo "    Untracked files:"

echo "  Size analysis:"
if [[ -d "cspice" ]]; then
    du -sh cspice/ 2>/dev/null | awk '{print "    cspice/: " $1}'
fi
if [[ -d "target" ]]; then
    du -sh target/ 2>/dev/null | awk '{print "    target/: " $1}'
else
    echo "    target/: not present (good!)"
fi

# What should be included
echo ""
echo "✅ What SHOULD be in repository:"
echo "  ✓ Rust source code (src/, tests/, benches/)"
echo "  ✓ CSPICE source files (.c, .h) for conversion reference"
echo "  ✓ CSPICE documentation for understanding algorithms"
echo "  ✓ Small test data files for validation"
echo "  ✓ Build scripts and configuration"
echo "  ✓ Project documentation"

echo ""
echo "❌ What should NOT be in repository:"
echo "  ✗ Build artifacts (target/, pkg/)"
echo "  ✗ CSPICE compiled libraries (.a, .so files)"
echo "  ✗ CSPICE executables"
echo "  ✗ Large binary data files"
echo "  ✗ Generated documentation (target/doc/)"

# Performance check
echo ""
echo "⚡ Performance Assessment:"
total_files=$(find . -type f 2>/dev/null | wc -l)
if [[ $total_files -gt 5000 ]]; then
    echo "  ⚠️  High file count ($total_files) - may impact IDE performance"
else
    echo "  ✅ Reasonable file count ($total_files)"
fi

large_files=$(find . -size +10M -not -path "./.git/*" 2>/dev/null | wc -l)
if [[ $large_files -gt 0 ]]; then
    echo "  ⚠️  Found $large_files large files (>10MB)"
    find . -size +10M -not -path "./.git/*" 2>/dev/null | head -5
else
    echo "  ✅ No unexpectedly large files"
fi

echo ""
echo "🎯 Recommendations:"
echo "  • Keep CSPICE source code - needed for Rust conversion"
echo "  • Keep core documentation files for algorithm reference"
echo "  • Exclude CSPICE binaries and executables"
echo "  • Monitor repository size - current approach is appropriate"
echo "  • The ~800 untracked files are likely CSPICE source - this is expected"
