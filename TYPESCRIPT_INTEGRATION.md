# TypeScript Integration Guide for RustSPICE

This document outlines the complete TypeScript integration strategy for RustSPICE, ensuring seamless interoperability between the Rust WASM module and TypeScript applications.

## Overview

RustSPICE provides first-class TypeScript support through:

1. **Automatic TypeScript Definition Generation** - Generated from Rust code using wasm-bindgen
2. **Type-Safe WASM Bindings** - Full TypeScript interface coverage
3. **JavaScript/TypeScript Interop** - Seamless data exchange
4. **NPM Package Distribution** - Ready-to-use TypeScript package
5. **IDE Support** - Full IntelliSense and auto-completion
6. **Testing Framework** - TypeScript-specific testing tools

## Architecture

```
┌─────────────────┐    ┌──────────────────┐    ┌─────────────────────┐
│   Rust Source   │    │   wasm-bindgen   │    │  TypeScript App     │
│                 │───▶│                  │───▶│                     │
│ - src/lib.rs    │    │ - Generates .d.ts│    │ - Full type safety  │
│ - WASM bindings │    │ - JavaScript glue│    │ - IntelliSense      │
│ - Type exports  │    │ - Type exports   │    │ - Auto-completion   │
└─────────────────┘    └──────────────────┘    └─────────────────────┘
```

## TypeScript Definition Generation

### Automatic Generation Process

1. **Rust Code Annotations**:
   ```rust
   #[wasm_bindgen]
   pub struct StateVector {
       pub x: f64,
       pub y: f64,
       pub z: f64,
       // ... other fields
   }
   
   #[wasm_bindgen]
   impl StateVector {
       #[wasm_bindgen(constructor)]
       pub fn new(x: f64, y: f64, z: f64, vx: f64, vy: f64, vz: f64, light_time: f64) -> StateVector {
           StateVector { x, y, z, vx, vy, vz, light_time }
       }
       
       #[wasm_bindgen]
       pub fn position(&self) -> Vec<f64> {
           vec![self.x, self.y, self.z]
       }
   }
   ```

2. **Generated TypeScript Definitions**:
   ```typescript
   export class StateVector {
       constructor(x: number, y: number, z: number, vx: number, vy: number, vz: number, light_time: number);
       readonly x: number;
       readonly y: number;
       readonly z: number;
       readonly vx: number;
       readonly vy: number;
       readonly vz: number;
       readonly light_time: number;
       position(): number[];
       velocity(): number[];
       magnitude(): number;
   }
   ```

### Enhanced Type Definitions

The build process creates comprehensive TypeScript definitions including:

- **Class Interfaces** - Full type coverage for Rust structs
- **Enum Definitions** - TypeScript enums for Rust enums
- **Function Signatures** - Complete parameter and return type information
- **Namespace Organization** - Logical grouping of related functions
- **Documentation Comments** - JSDoc comments from Rust documentation
- **Error Types** - Structured error handling with type safety

## Build Process

### 1. WASM Package Generation

```bash
# Build WASM package with TypeScript bindings
./wasm-pack-build.sh web release

# Output structure:
pkg/
├── rust_spice.js           # JavaScript glue code
├── rust_spice_bg.wasm      # WebAssembly binary
├── rust_spice.d.ts         # Generated TypeScript definitions
├── package.json            # NPM package configuration
├── README.md               # Usage documentation
└── example.ts              # TypeScript usage example
```

### 2. TypeScript Definition Enhancement

The build script creates enhanced definitions with:

```typescript
// Enhanced type definitions with namespaces
export namespace Time {
    export function calendar_to_et(year: number, month: number, day: number, 
                                   hour: number, minute: number, second: number): number;
    export function julian_date_to_et(julian_date: number): number;
    export function et_to_utc(et: number, precision?: number): string;
    export function utc_to_et(utc_string: string): number;
}

export namespace Kernels {
    export function load_kernel(data: Uint8Array, filename?: string): void;
    export function unload_kernel(filename: string): void;
    export function clear_kernels(): void;
    export function list_kernels(): string[];
}

export namespace Ephemeris {
    export function spkezr(target: string, et: number, reference_frame: string,
                          aberration_correction: string, observer: string): StateVector;
    export function spkpos(target: string, et: number, reference_frame: string,
                          aberration_correction: string, observer: string): 
                          { position: number[]; light_time: number };
}
```

## Usage Patterns

### 1. Browser Environment

```typescript
import init, { StateVector, Time, Ephemeris, Kernels } from '@rustspice/rust-spice';

async function main() {
    // Initialize WASM module
    await init();
    
    // Type-safe time conversion
    const et: number = Time.calendar_to_et(2000, 1, 1, 12, 0, 0);
    
    // Type-safe state vector creation
    const earthState: StateVector = new StateVector(
        149597870.7, 0, 0,  // Position (km)
        0, 29.78, 0,        // Velocity (km/s)
        0                   // Light time (s)
    );
    
    // Type-safe method calls
    const position: number[] = earthState.position();
    const magnitude: number = earthState.magnitude();
    
    // Load kernel with proper typing
    const kernelResponse = await fetch('/kernels/de421.bsp');
    const kernelData = new Uint8Array(await kernelResponse.arrayBuffer());
    Kernels.load_kernel(kernelData, 'de421.bsp');
    
    // Type-safe ephemeris calculation
    try {
        const marsState: StateVector = Ephemeris.spkezr(
            'MARS', et, 'J2000', 'LT+S', 'EARTH'
        );
        console.log('Mars position:', marsState.position());
    } catch (error) {
        console.error('Ephemeris calculation failed:', error);
    }
}
```

### 2. Node.js Environment

```typescript
import { readFileSync } from 'fs';
import init, { StateVector, Time, Kernels } from '@rustspice/rust-spice';

async function processEphemeris() {
    await init();
    
    // Load kernel from file
    const kernelData = readFileSync('./kernels/de421.bsp');
    Kernels.load_kernel(new Uint8Array(kernelData), 'de421.bsp');
    
    // Process multiple time points
    const timePoints: number[] = [];
    for (let day = 0; day < 365; day++) {
        const et = Time.calendar_to_et(2025, 1, 1 + day, 0, 0, 0);
        timePoints.push(et);
    }
    
    // Calculate ephemeris for each time point
    const results = timePoints.map(et => {
        const state = Ephemeris.spkezr('MARS', et, 'J2000', 'LT', 'EARTH');
        return {
            et,
            position: state.position(),
            distance: state.magnitude()
        };
    });
    
    return results;
}
```

### 3. React Application

```typescript
import React, { useEffect, useState } from 'react';
import init, { StateVector, Time } from '@rustspice/rust-spice';

interface EphemerisData {
    target: string;
    position: number[];
    velocity: number[];
    lightTime: number;
}

const EphemerisCalculator: React.FC = () => {
    const [wasmReady, setWasmReady] = useState(false);
    const [currentTime, setCurrentTime] = useState<number>(0);
    const [ephemerisData, setEphemerisData] = useState<EphemerisData | null>(null);
    
    useEffect(() => {
        init().then(() => {
            setWasmReady(true);
            const et = Time.calendar_to_et(2025, 7, 22, 15, 30, 0);
            setCurrentTime(et);
        });
    }, []);
    
    const calculateEphemeris = () => {
        if (!wasmReady) return;
        
        try {
            // In a real app, you would call Ephemeris.spkezr here
            // For now, create a dummy state vector
            const state = new StateVector(1000, 2000, 3000, 10, 20, 30, 0.5);
            
            setEphemerisData({
                target: 'MARS',
                position: state.position(),
                velocity: state.velocity(),
                lightTime: state.light_time
            });
        } catch (error) {
            console.error('Calculation failed:', error);
        }
    };
    
    return (
        <div>
            <h1>RustSPICE Ephemeris Calculator</h1>
            <p>WASM Status: {wasmReady ? '✅ Ready' : '⏳ Loading...'}</p>
            <p>Current ET: {currentTime.toFixed(2)} seconds past J2000</p>
            
            <button onClick={calculateEphemeris} disabled={!wasmReady}>
                Calculate Mars Position
            </button>
            
            {ephemerisData && (
                <div>
                    <h3>{ephemerisData.target} Ephemeris</h3>
                    <p>Position: [{ephemerisData.position.map(x => x.toFixed(2)).join(', ')}] km</p>
                    <p>Velocity: [{ephemerisData.velocity.map(x => x.toFixed(3)).join(', ')}] km/s</p>
                    <p>Light Time: {ephemerisData.lightTime.toFixed(3)} seconds</p>
                </div>
            )}
        </div>
    );
};

export default EphemerisCalculator;
```

## Testing Strategy

### 1. TypeScript Type Testing

```typescript
// Type-only tests to verify TypeScript definitions
import { StateVector, Time, SpiceErrorType } from '@rustspice/rust-spice';

// Compile-time type checking
const _testTypes = () => {
    // Test StateVector types
    const state: StateVector = new StateVector(1, 2, 3, 4, 5, 6, 7);
    const position: number[] = state.position();
    const magnitude: number = state.magnitude();
    
    // Test Time function types
    const et: number = Time.calendar_to_et(2000, 1, 1, 12, 0, 0);
    const utc: string = Time.et_to_utc(et, 3);
    
    // Test enum types
    const errorType: SpiceErrorType = SpiceErrorType.KernelNotFound;
    
    // Test optional parameters
    const utc1: string = Time.et_to_utc(et);        // No precision
    const utc2: string = Time.et_to_utc(et, 5);     // With precision
};
```

### 2. Runtime Integration Testing

```typescript
import { describe, it, expect, beforeAll } from '@jest/globals';
import init, { StateVector, Time } from '@rustspice/rust-spice';

describe('RustSPICE TypeScript Integration', () => {
    beforeAll(async () => {
        await init();
    });
    
    it('should maintain type safety at runtime', () => {
        const state = new StateVector(1, 2, 3, 4, 5, 6, 7);
        
        // Runtime type verification
        expect(typeof state.x).toBe('number');
        expect(Array.isArray(state.position())).toBe(true);
        expect(state.position()).toHaveLength(3);
    });
    
    it('should handle TypeScript error types correctly', () => {
        try {
            Time.utc_to_et('invalid format');
        } catch (error) {
            expect(error).toBeInstanceOf(Error);
            expect(typeof error.message).toBe('string');
        }
    });
});
```

### 3. Performance Testing

```typescript
import { performance } from 'perf_hooks';

async function benchmarkTypeScript() {
    await init();
    
    const iterations = 10000;
    const start = performance.now();
    
    for (let i = 0; i < iterations; i++) {
        const state = new StateVector(i, i*2, i*3, 1, 2, 3, 0);
        const position = state.position();
        const magnitude = state.magnitude();
    }
    
    const end = performance.now();
    console.log(`TypeScript operations: ${(end - start).toFixed(2)}ms`);
    console.log(`Rate: ${(iterations / (end - start) * 1000).toFixed(0)} ops/sec`);
}
```

## Development Workflow

### 1. Development Setup

```bash
# Clone repository
git clone https://github.com/Jthora/RustSPICE.git
cd RustSPICE

# Install dependencies
npm install

# Build WASM with TypeScript bindings
npm run build-wasm-dev

# Run TypeScript type checking
npm run test-ts

# Run comprehensive tests
npm run test-all
```

### 2. Hot Reload Development

```bash
# Watch for changes and rebuild
cargo watch -x "build --target wasm32-unknown-unknown" -s "npm run build-wasm-dev"

# TypeScript type checking in watch mode
npx tsc --noEmit --watch
```

### 3. Production Build

```bash
# Build optimized WASM package
npm run build-wasm

# Run all tests including TypeScript
npm run test-all

# Generate documentation
npm run docs
```

## IDE Configuration

### Visual Studio Code

```json
// .vscode/settings.json
{
    "typescript.preferences.importModuleSpecifier": "relative",
    "typescript.suggest.autoImports": true,
    "typescript.updateImportsOnFileMove.enabled": "always",
    "files.associations": {
        "*.wasm": "binary"
    },
    "rust-analyzer.linkedProjects": ["Cargo.toml"],
    "rust-analyzer.cargo.features": ["wasm-bindgen"]
}
```

### IntelliJ/WebStorm

```xml
<!-- Enable WASM support -->
<component name="ProjectCodeStyleConfiguration">
  <option name="WASM_SUPPORT" value="true" />
</component>
```

## Error Handling

### TypeScript Error Types

```typescript
// Structured error handling
export enum SpiceErrorType {
    KernelNotFound = "KernelNotFound",
    InvalidTime = "InvalidTime",
    InvalidTarget = "InvalidTarget",
    ComputationError = "ComputationError",
}

export class SpiceError extends Error {
    constructor(
        public readonly error_type: SpiceErrorType,
        message: string
    ) {
        super(message);
        this.name = 'SpiceError';
    }
}

// Usage with type safety
try {
    const state = Ephemeris.spkezr('INVALID', et, 'J2000', 'LT', 'EARTH');
} catch (error) {
    if (error instanceof SpiceError) {
        switch (error.error_type) {
            case SpiceErrorType.KernelNotFound:
                console.error('Missing kernel data:', error.message);
                break;
            case SpiceErrorType.InvalidTarget:
                console.error('Invalid target body:', error.message);
                break;
            default:
                console.error('SPICE error:', error.message);
        }
    } else {
        console.error('Unexpected error:', error);
    }
}
```

## Performance Considerations

### 1. Memory Management

```typescript
// Efficient memory usage
const processLargeDataset = async (timePoints: number[]) => {
    await init();
    
    // Process in batches to avoid memory pressure
    const batchSize = 1000;
    const results: StateVector[] = [];
    
    for (let i = 0; i < timePoints.length; i += batchSize) {
        const batch = timePoints.slice(i, i + batchSize);
        const batchResults = batch.map(et => 
            Ephemeris.spkezr('MARS', et, 'J2000', 'LT', 'EARTH')
        );
        results.push(...batchResults);
        
        // Allow garbage collection between batches
        if (i % (batchSize * 10) === 0) {
            await new Promise(resolve => setTimeout(resolve, 0));
        }
    }
    
    return results;
};
```

### 2. Optimization Techniques

```typescript
// Reuse objects when possible
class EphemerisCalculator {
    private cachedStates = new Map<string, StateVector>();
    
    getState(target: string, et: number): StateVector {
        const key = `${target}_${et}`;
        
        if (!this.cachedStates.has(key)) {
            const state = Ephemeris.spkezr(target, et, 'J2000', 'LT', 'EARTH');
            this.cachedStates.set(key, state);
        }
        
        return this.cachedStates.get(key)!;
    }
}
```

## Deployment

### 1. NPM Package

```bash
# Build and publish
npm run build-wasm
cd pkg
npm publish --access public
```

### 2. CDN Deployment

```html
<!-- Direct browser usage -->
<script type="module">
    import init, { StateVector, Time } from 'https://unpkg.com/@rustspice/rust-spice/rust_spice.js';
    
    async function main() {
        await init();
        const et = Time.calendar_to_et(2025, 7, 22, 15, 30, 0);
        console.log('Current ET:', et);
    }
    
    main();
</script>
```

### 3. Webpack Integration

```javascript
// webpack.config.js
module.exports = {
    experiments: {
        asyncWebAssembly: true,
    },
    resolve: {
        extensions: ['.ts', '.js', '.wasm'],
    },
    module: {
        rules: [
            {
                test: /\.wasm$/,
                type: 'asset/resource',
            },
        ],
    },
};
```

## Future Enhancements

1. **Worker Thread Support** - Run WASM in web workers for non-blocking calculations
2. **Streaming Data Processing** - Handle large datasets efficiently
3. **GPU Acceleration** - WebGL integration for parallel computations
4. **Real-time Updates** - Live ephemeris calculations with WebSocket data
5. **Progressive Loading** - Lazy-load WASM modules and kernels on demand

This comprehensive TypeScript integration ensures that RustSPICE provides a seamless, type-safe, and performant experience for JavaScript and TypeScript developers while maintaining the accuracy and reliability of the original NASA SPICE toolkit.
