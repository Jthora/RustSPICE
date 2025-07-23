#!/bin/bash

# RustSPICE Performance Optimization Verification Script

echo "🔍 RustSPICE Performance Optimization Check"
echo "==========================================="

# Check if optimization files exist
echo "✅ Checking optimization files..."
files=(".gitignore" ".copilotignore" ".vscode/settings.json" ".editorconfig")
for file in "${files[@]}"; do
    if [[ -f "$file" ]]; then
        echo "  ✓ $file exists"
    else
        echo "  ❌ $file missing"
    fi
done

# Check directory sizes
echo ""
echo "📊 Directory sizes:"
if [[ -d "target" ]]; then
    target_size=$(du -sh target/ 2>/dev/null | cut -f1)
    echo "  target/: $target_size"
    if [[ $(du -sb target/ 2>/dev/null | cut -f1) -gt 1000000000 ]]; then
        echo "  ⚠️  target/ is >1GB - consider running 'cargo clean'"
    fi
else
    echo "  target/: not present (good!)"
fi

if [[ -d "pkg" ]]; then
    pkg_size=$(du -sh pkg/ 2>/dev/null | cut -f1)
    echo "  pkg/: $pkg_size"
else
    echo "  pkg/: not present"
fi

if [[ -d "cspice" ]]; then
    cspice_size=$(du -sh cspice/ 2>/dev/null | cut -f1)
    echo "  cspice/: $cspice_size"
fi

# Check Git status
echo ""
echo "📋 Git status:"
untracked=$(git status --porcelain 2>/dev/null | grep "^??" | wc -l)
if [[ $untracked -gt 20 ]]; then
    echo "  ⚠️  $untracked untracked files - check .gitignore"
else
    echo "  ✓ $untracked untracked files (reasonable)"
fi

# Check for large files
echo ""
echo "🔍 Checking for large files (>10MB):"
large_files=$(find . -size +10M -not -path "./.git/*" -not -path "./cspice/cspice.tar" 2>/dev/null)
if [[ -z "$large_files" ]]; then
    echo "  ✓ No unexpected large files found"
else
    echo "  ⚠️  Large files found:"
    echo "$large_files" | sed 's/^/    /'
fi

# Check VS Code settings
echo ""
echo "⚙️  VS Code optimization:"
if [[ -f ".vscode/settings.json" ]]; then
    excluded_count=$(grep -c "exclude" .vscode/settings.json 2>/dev/null || echo "0")
    echo "  ✓ $excluded_count exclusion patterns configured"
else
    echo "  ❌ VS Code settings missing"
fi

# Performance recommendations
echo ""
echo "💡 Performance recommendations:"
echo "  • Run 'cargo clean' periodically to clear build cache"
echo "  • Use VS Code workspace file for better performance"
echo "  • Monitor file watcher count in VS Code status bar"
echo "  • Restart VS Code if it becomes slow"

echo ""
echo "✅ Optimization check complete!"
