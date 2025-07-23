# CSPICE Integration Plan

## Overview
Now that we have the official CSPICE source code from NASA/NAIF, we can implement our RustSPICE conversion strategy. The CSPICE code is extremely well-documented with excellent comments from the NAIF developers.

## CSPICE Structure Analysis

### Key Data Types (from SpiceZdf.h)
```c
typedef char           SpiceChar;
typedef double         SpiceDouble;
typedef float          SpiceFloat;
typedef int            SpiceInt;      // 32-bit on most platforms
typedef int            SpiceBoolean;  // SPICETRUE=1, SPICEFALSE=0
```

### Core Functions Found
1. **spkezr_c.c** - State vector calculation (position + velocity)
2. **furnsh_c.c** - Load SPICE kernel files
3. **failed_c.c** - Error checking
4. **getmsg_c.c** - Error message retrieval
5. **reset_c.c** - Reset error status

### C-to-FORTRAN Interface Pattern
CSPICE functions follow this pattern:
```c
// C wrapper function
void spkezr_c(ConstSpiceChar *targ, SpiceDouble et, /*...*/) {
    // Input validation using CHKFSTR, CHKIN_C
    // Call FORTRAN function via f2c
    spkezr_( (char*) targ, (doublereal*) &et, /*...*/, 
             (ftnlen) strlen(targ), /*...*/);  // Hidden string lengths!
    // Output validation, error checking
}
```

**Key Insight**: FORTRAN functions have hidden string length parameters!

## Implementation Strategy

### Phase 1: Hybrid Approach (Immediate Goal)
Compile CSPICE to WASM and create Rust bindings.

#### Step 1: Set up Emscripten Build
```bash
# Install Emscripten SDK
git clone https://github.com/emscripten-core/emsdk.git
cd emsdk
./emsdk install latest
./emsdk activate latest
source ./emsdk_env.sh
```

#### Step 2: Compile CSPICE to WASM
The challenge: CSPICE includes both C and FORTRAN code. We need:
1. **Emscripten** for C code (readily available)
2. **LLVM Flang with WASM patches** for FORTRAN code

#### Step 3: Key Functions to Export
```bash
emcc -o cspice.wasm \
  -s EXPORTED_FUNCTIONS='["_spkezr_c","_furnsh_c","_failed_c","_getmsg_c","_reset_c","_kclear_c"]' \
  -s ALLOW_MEMORY_GROWTH=1 \
  cspice_sources...
```

### Phase 2: Virtual Filesystem for Kernels
CSPICE expects file paths, but WASM can't access files directly.

**Solution**: Use Emscripten's FS (virtual filesystem):
```javascript
// Load kernel file as ArrayBuffer
const kernelData = await fetch('de442.bsp').then(r => r.arrayBuffer());

// Mount in virtual filesystem  
FS.writeFile('/kernels/de442.bsp', new Uint8Array(kernelData));

// Now CSPICE can access it
Module.ccall('furnsh_c', null, ['string'], ['/kernels/de442.bsp']);
```

### Phase 3: Update Rust Bindings
Replace our current extern declarations with actual WASM imports:

```rust
// Current (stub)
#[wasm_bindgen]
extern "C" {
    fn spkezr_c(targ: *const c_char, et: f64, ...);
}

// Updated (real WASM import)
#[wasm_bindgen(module = "/pkg/cspice.js")]
extern "C" {
    fn spkezr_c(targ: *const c_char, et: f64, ...);
}
```

## Directory Structure
```
RustSPICE/
├── cspice/                    # Official CSPICE source (✅ DONE)
│   └── cspice/
│       ├── src/cspice/        # C wrapper functions
│       ├── include/           # Header files  
│       └── lib/               # Pre-built libraries
├── cspice-wasm/              # WASM compilation artifacts
│   ├── build.sh              # Emscripten build script
│   ├── cspice.wasm           # Compiled WASM module
│   └── cspice.js             # JS glue code
├── src/
│   ├── lib.rs                # Rust WASM interface (✅ DONE)
│   ├── cspice_bindings.rs    # Raw CSPICE function bindings
│   ├── kernel_loader.rs      # Virtual filesystem management
│   └── spice_types.rs        # Type conversions
└── tests/
    ├── integration/          # Test with real ephemeris data
    └── kernels/              # Test kernel files
```

## Next Steps

### Immediate (Next Session)
1. **Set up Emscripten environment**
2. **Create minimal CSPICE WASM build** (just spkezr_c + furnsh_c)
3. **Test virtual filesystem approach** for kernel loading
4. **Update Rust bindings** to call real WASM functions

### Key Challenges to Address
1. **FORTRAN Hidden Arguments**: String lengths passed as hidden parameters
2. **Memory Layout**: Ensure 32-bit WASM compatibility with CSPICE assumptions  
3. **Error Handling**: Integrate CSPICE error system with Rust Result types
4. **Virtual FS**: Efficient kernel file loading in browser environment

### Success Metrics
- [ ] Load a real .bsp file (e.g., de442.bsp) in browser
- [ ] Calculate Earth-Moon distance for a known date
- [ ] Validate result matches native CSPICE within machine precision
- [ ] Create TypeScript interface for web developers

## Resources
- **CSPICE Documentation**: `/cspice/cspice/doc/`
- **Function Reference**: Each .c file has extensive header comments
- **Example Kernels**: Available from NAIF website
- **Emscripten FS Documentation**: https://emscripten.org/docs/api_reference/Filesystem-API.html

The CSPICE source code is remarkably well-documented with clear comments from the NASA/NAIF team. This gives us a solid foundation for understanding the algorithms and creating a faithful Rust implementation.
