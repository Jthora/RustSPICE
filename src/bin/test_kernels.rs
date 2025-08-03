use rust_spice::*;
use rust_spice::kernel_system::{initialize_kernel_system, furnish_kernel_from_bytes};
use rust_spice::kernel_pool::initialize_pool;
use std::fs;

fn main() -> SpiceResult<()> {
    println!("=== RustSPICE Kernel Test ===");
    
    // Initialize systems
    println!("1. Initializing kernel system...");
    initialize_kernel_system()?;
    
    println!("2. Initializing kernel pool...");
    initialize_pool()?;
    
    println!("3. Initializing SPK reader...");
    spk_reader::initialize_spk_reader()?;
    
    // Load kernels from disk into virtual file system
    println!("4. Loading kernels into virtual file system...");
    let kernel_files = [
        ("kernels/spk/de442.bsp", "/kernels/spk/de442.bsp"),
        ("kernels/lsk/naif0012.tls", "/kernels/lsk/naif0012.tls"), 
        ("kernels/pck/pck00011.tpc", "/kernels/pck/pck00011.tpc"),
    ];
    
    for (disk_path, vfs_path) in &kernel_files {
        match fs::read(disk_path) {
            Ok(data) => {
                match furnish_kernel_from_bytes(data, vfs_path) {
                    Ok(_) => println!("   ✓ Loaded: {} into VFS as {}", disk_path, vfs_path),
                    Err(e) => println!("   ✗ Failed to furnish {}: {}", vfs_path, e),
                }
            },
            Err(e) => println!("   ✗ Failed to read {}: {}", disk_path, e),
        }
    }
    
    // Try a simple ephemeris calculation
    println!("5. Testing ephemeris calculation...");
    let et = str_to_et("2025-01-01T12:00:00")?;
    println!("   Converted time: {:.2} ET", et.0);
    
    // Test various body combinations to see what's available
    let test_cases = [
        ("Mars barycenter to Solar System barycenter", "4", "0"),
        ("Earth barycenter to Solar System barycenter", "3", "0"), 
        ("Mars to Earth", "499", "399"),
        ("Earth to Mars", "399", "499"),
        ("Mars barycenter to Earth barycenter", "4", "3"),
    ];
    
    for (description, target, observer) in test_cases {
        match ephemeris_position(target, et, "J2000", "NONE", observer) {
            Ok(pos) => {
                println!("   ✓ {}: ({:.2}, {:.2}, {:.2}) km", 
                         description, pos.x(), pos.y(), pos.z());
                println!("     Distance: {:.2} km", pos.magnitude());
                break; // Success! Exit the loop
            },
            Err(e) => println!("   ✗ {}: {}", description, e),
        }
    }
    
    println!("=== Test Complete ===");
    Ok(())
}
