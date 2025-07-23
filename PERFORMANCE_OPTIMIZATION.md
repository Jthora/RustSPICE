# Performance Optimization Guide

## Overview
This project has been optimized for GitHub Copilot, VS Code performance, and Git operations to prevent slowdowns and crashes.

## What's Been Optimized

### 1. Git Configuration
- **`.gitignore`**: Excludes build artifacts, binaries, and large files
- **`.dockerignore`**: Optimized for container builds
- Large directories excluded:
  - `target/` (Rust build artifacts - can be 1GB+)
  - `pkg/` (WASM build output)
  - `cspice/cspice/lib/` and `cspice/cspice/exe/` (compiled binaries)

### 2. VS Code Optimization
- **`.vscode/settings.json`**: 
  - File watcher exclusions for performance
  - Search exclusions to prevent indexing large files
  - Rust-analyzer optimizations
- **`.vscode/extensions.json`**: Recommended extensions
- **`.vscode/tasks.json`**: Pre-configured build and test tasks
- **`RustSPICE.code-workspace`**: Workspace configuration

### 3. GitHub Copilot Optimization
- **`.copilotignore`**: Excludes large files from AI context
- VS Code settings optimized for Copilot performance
- Binary files and build artifacts excluded from indexing

### 4. Build Performance
- **Cargo.toml profiles optimized**:
  - Release builds for small size (`opt-level = "s"`)
  - Development dependencies optimized (`opt-level = 1`)
  - LTO and strip enabled for production

## Performance Impact

### Before Optimization
- `target/` directory: **1.4GB** (3,321 files)
- `cspice/` directory: **233MB** (4,149 files)
- No Git ignores - risk of tracking binaries
- No VS Code exclusions - full indexing of all files

### After Optimization
- Build artifacts properly excluded
- File watchers optimized
- Search indexing limited to source code
- Copilot context focused on relevant files

## Usage Guidelines

### Development Workflow
```bash
# Clean builds when needed
cargo clean

# Use VS Code tasks (Ctrl+Shift+P > "Tasks: Run Task")
- "Rust: cargo build"
- "Rust: cargo test" 
- "WASM: build with wasm-pack"
- "Clean: cargo clean"
```

### File Organization
- Keep source code in `src/`
- Tests in `tests/`
- Documentation in root (but avoid large files)
- Build scripts are OK in root

### What to Avoid
- Don't commit files in `target/` or `pkg/`
- Don't commit large binary files
- Don't edit files in `cspice/cspice/` (except data files if needed)
- Avoid creating large log files in the workspace

## Monitoring Performance

### VS Code Performance
- Check "Developer: Show Running Extensions" if slow
- Use "Developer: Reload Window" if VS Code becomes unresponsive
- Monitor file watcher count in VS Code status bar

### Git Performance
```bash
# Check repository size
git count-objects -vH

# Check what's taking space
git ls-files | xargs ls -la | sort -k5 -nr | head -20
```

### Build Performance
```bash
# Time builds
time cargo build --release

# Check target directory size
du -sh target/
```

## Troubleshooting

### VS Code Slow/Crashing
1. Reload window (Ctrl+Shift+P > "Developer: Reload Window")
2. Check excluded files are still excluded in settings
3. Run `cargo clean` to remove build artifacts

### Copilot Not Working Well
1. Check `.copilotignore` is present
2. Ensure large files aren't being indexed
3. Restart Copilot extension

### Git Operations Slow
1. Verify `.gitignore` is working: `git status`
2. Clean untracked files: `git clean -fd`
3. Check for large files: `git ls-files | xargs ls -la | sort -k5 -nr | head -10`

## Maintenance

Run this monthly to keep the workspace clean:
```bash
# Clean all build artifacts
cargo clean

# Remove any accidentally created large files
find . -size +10M -not -path "./.git/*" -not -path "./cspice/cspice.tar"

# Check Git status
git status
```
