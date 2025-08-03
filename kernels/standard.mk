\begindata

KERNELS_TO_LOAD = (
                   'spk/de442.bsp',
                   'lsk/naif0012.tls',
                   'pck/pck00011.tpc'
                  )

\begintext

RustSPICE Standard Kernel Set
=============================

This meta-kernel loads the essential SPICE kernels for basic ephemeris
calculations and time conversions:

1. DE442 Planetary Ephemeris (SPK)
   - High-precision planetary positions from 1550 to 2650 CE
   - Includes positions of planets, Moon, and major asteroids

2. NAIF Leap Second Kernel (LSK) 
   - Current leap second definitions for UTC/ET conversions
   - Updated regularly by JPL NAIF team

3. Planetary Constants Kernel (PCK)
   - Physical and orientation constants for planets and satellites
   - Required for body-fixed coordinate transformations

Usage:
------
To load these kernels in your RustSPICE application:

```rust
use rust_spice::*;

// Load the standard kernel set
furnish_kernel("kernels/standard.mk")?;

// Now you can perform ephemeris calculations
let et = str_to_et("2025-07-27T12:00:00")?;
let state = ephemeris_state("MARS", et, "J2000", "LT+S", "EARTH")?;
```

For test environments, use relative paths:
```rust
furnish_kernel("../../kernels/standard.mk")?;
```
