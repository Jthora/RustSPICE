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
