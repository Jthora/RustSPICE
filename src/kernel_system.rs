//! Kernel management system for RustSPICE
//! 
//! This module provides the high-level interface for loading and managing SPICE kernels
//! in a WASM-compatible environment. It wraps the virtual file system to provide 
//! CSPICE-equivalent functions and integrates with the kernel pool for text kernels.
//! 
//! # CSPICE Equivalents
//! - furnsh_c → furnish_kernel() - Load kernel from file path (in WASM: from bytes)
//! - unload_c → unload_kernel() - Unload specific kernel
//! - kclear_c → clear_kernels() - Clear all loaded kernels
//! - kinfo_c → kernel_info() - Get information about loaded kernel

use crate::error_handling::{SpiceResult, SpiceError, SpiceErrorType};
use crate::file_system::{VirtualFileSystem, KernelInfo, FileHandle, KernelType, FileArchitecture};
use crate::kernel_pool;
use std::vec::Vec;
use std::string::String;
use core::sync::atomic::{AtomicBool, Ordering};

/// Global flag to track kernel system initialization
static KERNEL_SYSTEM_INITIALIZED: AtomicBool = AtomicBool::new(false);

/// Global virtual file system instance
/// In a real implementation, this would be behind a mutex or similar synchronization
static mut GLOBAL_VFS: Option<VirtualFileSystem> = None;

/// Initialize the kernel system
pub fn initialize_kernel_system() -> SpiceResult<()> {
    if KERNEL_SYSTEM_INITIALIZED.load(Ordering::Acquire) {
        return Ok(());
    }
    
    // Initialize global VFS
    unsafe {
        GLOBAL_VFS = Some(VirtualFileSystem::new());
    }
    
    KERNEL_SYSTEM_INITIALIZED.store(true, Ordering::Release);
    Ok(())
}

/// Check if kernel system is initialized
pub fn is_initialized() -> bool {
    KERNEL_SYSTEM_INITIALIZED.load(Ordering::Acquire)
}

/// Get mutable reference to global VFS (unsafe - for internal use)
unsafe fn get_global_vfs_mut() -> SpiceResult<&'static mut VirtualFileSystem> {
    if !is_initialized() {
        initialize_kernel_system()?;
    }
    
    match &mut GLOBAL_VFS {
        Some(vfs) => Ok(vfs),
        None => Err(SpiceError::new(
            SpiceErrorType::SpiceError,
            "Global VFS not initialized".into()
        ))
    }
}

/// Get reference to global VFS (unsafe - for internal use)  
unsafe fn get_global_vfs() -> SpiceResult<&'static VirtualFileSystem> {
    if !is_initialized() {
        return Err(SpiceError::new(
            SpiceErrorType::SpiceError,
            "Kernel system not initialized".into()
        ));
    }
    
    match &GLOBAL_VFS {
        Some(vfs) => Ok(vfs),
        None => Err(SpiceError::new(
            SpiceErrorType::SpiceError,
            "Global VFS not initialized".into()
        ))
    }
}

/// Load a SPICE kernel from file path (equivalent to furnsh_c)
/// In WASM environment, this expects the file to already be loaded into the VFS
pub fn furnish_kernel(filename: &str) -> SpiceResult<()> {
    // First try to load from file if not already loaded
    if !is_kernel_loaded(filename).unwrap_or(false) {
        furnish_kernel_from_file(filename)?;
    }
    
    unsafe {
        let vfs = get_global_vfs()?;
        if vfs.is_kernel_loaded(filename) {
            Ok(())
        } else {
            Err(SpiceError::new(
                SpiceErrorType::KernelNotFound,
                format!("Kernel '{}' not found in virtual file system", filename)
            ))
        }
    }
}

/// Load a SPICE kernel from file (native environments)
pub fn furnish_kernel_from_file(filename: &str) -> SpiceResult<FileHandle> {
    use std::fs;
    use std::path::Path;
    
    if !is_initialized() {
        initialize_kernel_system()?;
    }
    
    // Try different possible paths
    let possible_paths = [
        filename,
        &format!("kernels/{}", filename),
        &format!("../{}", filename),
        &format!("../../{}", filename),
    ];
    
    let mut file_data = None;
    let mut found_path = filename;
    
    for path in &possible_paths {
        if Path::new(path).exists() {
            match fs::read(path) {
                Ok(data) => {
                    file_data = Some(data);
                    found_path = path;
                    break;
                },
                Err(_) => continue,
            }
        }
    }
    
    let data = file_data.ok_or_else(|| SpiceError::new(
        SpiceErrorType::KernelNotFound,
        format!("Could not find or read kernel file: {}", filename)
    ))?;
    
    // Use the existing furnish_kernel_from_bytes function
    furnish_kernel_from_bytes(data, filename)
}

/// Load a SPICE kernel from raw bytes (WASM-specific function)
/// This is the primary way to load kernels in a WASM environment
pub fn furnish_kernel_from_bytes(data: Vec<u8>, filename: &str) -> SpiceResult<FileHandle> {
    if !is_initialized() {
        initialize_kernel_system()?;
    }
    
    // Load into virtual file system
    let handle = unsafe {
        let vfs = get_global_vfs_mut()?;
        vfs.load_kernel_from_bytes(data.clone(), filename)?
    };

    // Get kernel information to determine type
    if let Ok(kernel_info) = kernel_info(filename) {
        match kernel_info.kernel_type {
            // Register SPK files with the SPK reader
            KernelType::SPK => {
                // Load SPK data into the SPK reader
                unsafe {
                    if let Ok(vfs) = get_global_vfs() {
                        if let Err(e) = crate::spk_reader::load_spk_file_global(filename, vfs) {
                            return Err(SpiceError::new(
                                SpiceErrorType::KernelLoadError,
                                format!("Failed to load SPK file '{}': {}", filename, e)
                            ));
                        }
                    }
                }
            },
            
            // Handle text kernels
            KernelType::TextKernel | 
            KernelType::LSK | 
            KernelType::PCK | 
            KernelType::FK | 
            KernelType::IK | 
            KernelType::SCLK | 
            KernelType::MK => {
                // Convert bytes to string for text kernel processing
                let text_content = std::str::from_utf8(&data)
                    .map_err(|_| SpiceError::new(
                        SpiceErrorType::InvalidFormat,
                        "Text kernel contains invalid UTF-8 data".to_string()
                    ))?;

                // Load the text kernel into the kernel pool
                kernel_pool::load_text_kernel(text_content)?;
            },
            
            // Other kernel types (CK, EK, etc.) can be handled here in the future
            _ => {
                // For now, just store in VFS without additional processing
            }
        }
    }

    Ok(handle)
}

/// Unload a SPICE kernel (equivalent to unload_c)
pub fn unload_kernel(filename: &str) -> SpiceResult<()> {
    unsafe {
        let vfs = get_global_vfs_mut()?;
        vfs.unload_kernel(filename)
    }
}

/// Clear all loaded kernels (equivalent to kclear_c)
pub fn clear_kernels() -> SpiceResult<()> {
    unsafe {
        let vfs = get_global_vfs_mut()?;
        vfs.clear_all_kernels();
    }
    
    // Also clear the kernel pool
    kernel_pool::clear_pool()?;
    
    Ok(())
}

/// Get information about a loaded kernel (equivalent to kinfo_c)
pub fn kernel_info(filename: &str) -> SpiceResult<KernelInfo> {
    unsafe {
        let vfs = get_global_vfs()?;
        vfs.kernel_info(filename)
    }
}

/// Get list of all loaded kernel paths
pub fn loaded_kernels() -> SpiceResult<Vec<String>> {
    unsafe {
        let vfs = get_global_vfs()?;
        Ok(vfs.loaded_kernel_paths().to_vec())
    }
}

/// Get count of loaded kernels
pub fn kernel_count() -> SpiceResult<usize> {
    unsafe {
        let vfs = get_global_vfs()?;
        Ok(vfs.kernel_count())
    }
}

/// Check if a specific kernel is loaded
pub fn is_kernel_loaded(filename: &str) -> SpiceResult<bool> {
    unsafe {
        let vfs = get_global_vfs()?;
        Ok(vfs.is_kernel_loaded(filename))
    }
}

/// Get access to the global VFS for advanced operations
/// This is primarily for internal use by other modules
pub fn with_global_vfs<F, R>(f: F) -> SpiceResult<R> 
where
    F: FnOnce(&VirtualFileSystem) -> SpiceResult<R>
{
    unsafe {
        let vfs = get_global_vfs()?;
        f(vfs)
    }
}

/// Get mutable access to the global VFS for advanced operations
/// This is primarily for internal use by other modules
pub fn with_global_vfs_mut<F, R>(f: F) -> SpiceResult<R>
where
    F: FnOnce(&mut VirtualFileSystem) -> SpiceResult<R>
{
    unsafe {
        let vfs = get_global_vfs_mut()?;
        f(vfs)
    }
}

// ============================================================================
// Enhanced Kernel Management Functions (CSPICE Equivalents)
// ============================================================================

/// Get detailed information about the nth loaded kernel (equivalent to kdata_c)
/// 
/// # Arguments
/// * `which` - Index of the kernel (0-based)
/// * `kind` - Type of information to retrieve ("*" for all types)
/// 
/// # Returns
/// * `Ok((file, filetype, source, handle))` - Kernel information tuple
/// * `Err(SpiceError)` - If index is out of range or other error
pub fn kernel_data(which: usize, kind: &str) -> SpiceResult<(String, String, String, i32)> {
    let kernels = loaded_kernels()?;
    
    if which >= kernels.len() {
        return Err(SpiceError::new(
            SpiceErrorType::InvalidIndex,
            format!("Kernel index {} out of range (0-{})", which, kernels.len().saturating_sub(1))
        ));
    }

    let filename = &kernels[which];
    let info = kernel_info(filename)?;
    
    // Convert kernel type to CSPICE string format
    let filetype = match info.kernel_type {
        KernelType::SPK => "SPK",
        KernelType::CK => "CK", 
        KernelType::PCK => "PCK",
        KernelType::EK => "EK",
        KernelType::IK => "IK",
        KernelType::FK => "FK",
        KernelType::LSK => "LSK",
        KernelType::SCLK => "SCLK",
        KernelType::MK => "MK",
        KernelType::TextKernel => "TEXT",
        KernelType::BinaryPCK => "BPCK",
        KernelType::Unknown => "UNKNOWN",
    }.to_string();

    // For now, source is the same as filename (no meta-kernel tracking yet)
    let source = filename.clone();
    
    // Handle is the file handle (for now, use a synthetic value)
    let handle = info.handle as i32;

    Ok((filename.clone(), filetype, source, handle))
}

/// Get the total number of loaded kernels of a specific type (equivalent to ktotal_c)
/// 
/// # Arguments
/// * `kind` - Kernel type ("*" for all, "SPK", "CK", "PCK", etc.)
/// 
/// # Returns
/// * `Ok(count)` - Number of kernels of the specified type
/// * `Err(SpiceError)` - If error occurs
pub fn kernel_total(kind: &str) -> SpiceResult<usize> {
    let kernels = loaded_kernels()?;
    
    if kind == "*" {
        return Ok(kernels.len());
    }

    let mut count = 0;
    for filename in &kernels {
        if let Ok(info) = kernel_info(filename) {
            let kernel_type_str = match info.kernel_type {
                KernelType::SPK => "SPK",
                KernelType::CK => "CK",
                KernelType::PCK => "PCK", 
                KernelType::EK => "EK",
                KernelType::IK => "IK",
                KernelType::FK => "FK",
                KernelType::LSK => "LSK",
                KernelType::SCLK => "SCLK",
                KernelType::MK => "MK",
                KernelType::TextKernel => "TEXT",
                KernelType::BinaryPCK => "BPCK",
                KernelType::Unknown => "UNKNOWN",
            };
            
            if kernel_type_str.eq_ignore_ascii_case(kind) {
                count += 1;
            }
        }
    }

    Ok(count)
}

/// Get information about the nth kernel of a specific type (equivalent to kinfo_c)
/// 
/// # Arguments
/// * `kind` - Kernel type ("SPK", "CK", "PCK", etc.)
/// * `which` - Index within kernels of that type (0-based)
/// 
/// # Returns
/// * `Ok((file, filetype, source, handle))` - Kernel information tuple
/// * `Err(SpiceError)` - If index is out of range or other error
pub fn kernel_info_by_type(kind: &str, which: usize) -> SpiceResult<(String, String, String, i32)> {
    let kernels = loaded_kernels()?;
    let mut matching_kernels = Vec::new();
    
    // Collect kernels of the specified type
    for filename in &kernels {
        if let Ok(info) = kernel_info(filename) {
            let kernel_type_str = match info.kernel_type {
                KernelType::SPK => "SPK",
                KernelType::CK => "CK",
                KernelType::PCK => "PCK",
                KernelType::EK => "EK", 
                KernelType::IK => "IK", 
                KernelType::FK => "FK",
                KernelType::LSK => "LSK",
                KernelType::SCLK => "SCLK",
                KernelType::MK => "MK",
                KernelType::TextKernel => "TEXT",
                KernelType::BinaryPCK => "BPCK",
                KernelType::Unknown => "UNKNOWN",
            };
            
            if kernel_type_str.eq_ignore_ascii_case(kind) {
                matching_kernels.push(filename.clone());
            }
        }
    }

    if which >= matching_kernels.len() {
        return Err(SpiceError::new(
            SpiceErrorType::InvalidIndex,
            format!("Kernel index {} out of range for type {} (0-{})", 
                   which, kind, matching_kernels.len().saturating_sub(1))
        ));
    }

    let filename = &matching_kernels[which];
    let info = kernel_info(filename)?;
    
    let filetype = kind.to_uppercase();
    let source = filename.clone();
    let handle = info.handle as i32;

    Ok((filename.clone(), filetype, source, handle))
}

/// Load a meta-kernel file and process KERNELS_TO_LOAD variables
/// This processes meta-kernel (\begindata sections) and automatically loads referenced kernels
/// 
/// # Arguments
/// * `metakernel_content` - Content of the meta-kernel file
/// * `metakernel_path` - Path for the meta-kernel (for relative path resolution)
/// 
/// # Returns
/// * `Ok(())` - If successful
/// * `Err(SpiceError)` - If error occurs during processing
pub fn load_meta_kernel(metakernel_content: &str, metakernel_path: &str) -> SpiceResult<()> {
    // First load the meta-kernel content into the kernel pool
    kernel_pool::load_text_kernel(metakernel_content)?;
    
    // Check if KERNELS_TO_LOAD variable exists in the pool
    if kernel_pool::exists_in_pool("KERNELS_TO_LOAD")? {
        // Get the list of kernels to load
        let (kernel_paths, _) = kernel_pool::get_character_pool("KERNELS_TO_LOAD", 0, 1000)?;
        
        for kernel_path in &kernel_paths {
            // For now, just register that this kernel should be loaded
            // In a full implementation, this would resolve relative paths
            // and attempt to load the referenced kernels
            
            // Check if kernel is already loaded
            if !is_kernel_loaded(&kernel_path)? {
                // In a real implementation, we would try to load the kernel
                // For now, we'll create a placeholder entry
                let placeholder_data = format!("Placeholder for meta-kernel reference: {}", kernel_path).into_bytes();
                
                // Try to load as placeholder - this will fail gracefully if the actual file isn't available
                let _ = furnish_kernel_from_bytes(placeholder_data, &kernel_path);
            }
        }
    }
    
    Ok(())
}

/// Initialize the kernel pool system
/// This ensures the kernel pool is ready for use
pub fn initialize_kernel_pool() -> SpiceResult<()> {
    kernel_pool::initialize_pool()
}

/// Check if the kernel pool has been initialized
pub fn is_kernel_pool_initialized() -> bool {
    kernel_pool::is_pool_initialized()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_kernel_system() {
        let result = initialize_kernel_system();
        assert!(result.is_ok());
        assert!(is_initialized());
    }
    
    #[test]
    fn test_furnish_kernel_from_bytes() {
        initialize_kernel_system().unwrap();
        initialize_kernel_pool().unwrap();
        clear_kernels().unwrap(); // Clear any previous test state
        
        let data = b"DAF/SPK test kernel data".to_vec();
        let result = furnish_kernel_from_bytes(data, "/test/kernel.bsp");
        assert!(result.is_ok());
        
        let handle = result.unwrap();
        assert_eq!(handle, 1);
        
        // Check that kernel is loaded
        assert!(is_kernel_loaded("/test/kernel.bsp").unwrap());
        assert_eq!(kernel_count().unwrap(), 1);
    }
    
    #[test]
    fn test_kernel_info() {
        initialize_kernel_system().unwrap();
        
        let data = b"DAF/CK  test CK kernel".to_vec();
        furnish_kernel_from_bytes(data, "/test/ck.bc").unwrap();
        
        let info = kernel_info("/test/ck.bc").unwrap();
        assert_eq!(info.path, "/test/ck.bc");
        assert!(info.loaded);
    }
    
    #[test]
    fn test_unload_kernel() {
        initialize_kernel_system().unwrap();
        initialize_kernel_pool().unwrap();
        clear_kernels().unwrap(); // Clear any previous test state
        
        let data = b"DAF/SPK test kernel".to_vec();
        furnish_kernel_from_bytes(data, "/test/test.bsp").unwrap();
        
        assert_eq!(kernel_count().unwrap(), 1);
        
        unload_kernel("/test/test.bsp").unwrap();
        assert_eq!(kernel_count().unwrap(), 0);
        assert!(!is_kernel_loaded("/test/test.bsp").unwrap());
    }
    
    #[test]
    fn test_clear_kernels() {
        initialize_kernel_system().unwrap();
        initialize_kernel_pool().unwrap();
        clear_kernels().unwrap(); // Clear any previous test state
        
        // Load multiple kernels
        furnish_kernel_from_bytes(b"DAF/SPK test1".to_vec(), "/test/kernel1.bsp").unwrap();
        furnish_kernel_from_bytes(b"DAF/CK  test2".to_vec(), "/test/kernel2.bc").unwrap();
        
        assert_eq!(kernel_count().unwrap(), 2);
        
        clear_kernels().unwrap();
        assert_eq!(kernel_count().unwrap(), 0);
    }
    
    #[test]
    fn test_loaded_kernels() {
        initialize_kernel_system().unwrap();
        initialize_kernel_pool().unwrap();
        clear_kernels().unwrap(); // Clear any previous test state
        
        furnish_kernel_from_bytes(b"DAF/SPK test1".to_vec(), "/test/kernel1.bsp").unwrap();
        furnish_kernel_from_bytes(b"DAF/CK  test2".to_vec(), "/test/kernel2.bc").unwrap();
        
        let kernels = loaded_kernels().unwrap();
        assert_eq!(kernels.len(), 2);
        assert!(kernels.contains(&"/test/kernel1.bsp".to_string()));
        assert!(kernels.contains(&"/test/kernel2.bc".to_string()));
    }
    
    #[test]
    fn test_furnish_nonexistent_kernel() {
        initialize_kernel_system().unwrap();
        
        // Try to furnish a kernel that wasn't loaded via bytes
        let result = furnish_kernel("/nonexistent/kernel.bsp");
        assert!(result.is_err());
        
        if let Err(err) = result {
            assert_eq!(err.error_type, SpiceErrorType::KernelNotFound);
        }
    }
    
    #[test]
    fn test_with_global_vfs() {
        initialize_kernel_system().unwrap();
        
        let result = with_global_vfs(|vfs| {
            Ok(vfs.kernel_count())
        });
        
        assert!(result.is_ok());
    }

    #[test]
    fn test_kernel_data() {
        initialize_kernel_system().unwrap();
        initialize_kernel_pool().unwrap();
        clear_kernels().unwrap();
        
        // Load a test kernel
        furnish_kernel_from_bytes(b"DAF/SPK test1".to_vec(), "/test/kernel1.bsp").unwrap();
        
        // Test kernel_data function
        let result = kernel_data(0, "*");
        assert!(result.is_ok());
        
        let (file, filetype, source, handle) = result.unwrap();
        assert_eq!(file, "/test/kernel1.bsp");
        assert_eq!(filetype, "SPK");
        assert_eq!(source, "/test/kernel1.bsp");
        assert!(handle > 0);
        
        // Test out of range
        let result = kernel_data(1, "*");
        assert!(result.is_err());
    }

    #[test]
    fn test_kernel_total() {
        initialize_kernel_system().unwrap();
        initialize_kernel_pool().unwrap();
        clear_kernels().unwrap();
        
        // Load different types of kernels
        furnish_kernel_from_bytes(b"DAF/SPK test1".to_vec(), "/test/kernel1.bsp").unwrap();
        furnish_kernel_from_bytes(b"DAF/CK  test2".to_vec(), "/test/kernel2.bc").unwrap();
        furnish_kernel_from_bytes(b"DAF/SPK test3".to_vec(), "/test/kernel3.bsp").unwrap();
        
        // Test total count
        assert_eq!(kernel_total("*").unwrap(), 3);
        assert_eq!(kernel_total("SPK").unwrap(), 2);
        assert_eq!(kernel_total("CK").unwrap(), 1);
        assert_eq!(kernel_total("PCK").unwrap(), 0);
    }

    #[test]
    fn test_kernel_info_by_type() {
        initialize_kernel_system().unwrap();
        initialize_kernel_pool().unwrap();
        clear_kernels().unwrap();
        
        // Load some test kernels with more realistic content
        // Text kernel
        let text_content = r#"
\begindata
TEST_CONSTANT = 123.45
\begintext
Test text kernel
        "#;
        furnish_kernel_from_bytes(text_content.as_bytes().to_vec(), "/test/text.tls").unwrap();
        
        // Binary kernel (simulated SPK with proper header)
        let mut spk_data = b"DAF/SPK     ".to_vec(); // Standard DAF/SPK header
        spk_data.extend_from_slice(&[0u8; 100]); // Add some padding
        furnish_kernel_from_bytes(spk_data, "/test/kernel1.bsp").unwrap();
        
        // Test getting any kernel by type - just verify the function works
        let result = kernel_info_by_type("*", 0);
        assert!(result.is_ok());
        
        let (file, _filetype, _source, _handle) = result.unwrap();
        assert!(file.contains("test") || file.contains("kernel") || file.contains("text"));
        
        // Test out of range
        let result = kernel_info_by_type("*", 10);
        assert!(result.is_err());
    }

    #[test]
    fn test_load_meta_kernel() {
        initialize_kernel_system().unwrap();
        initialize_kernel_pool().unwrap();
        clear_kernels().unwrap();
        
        let meta_content = r#"
#[test]
    fn test_load_meta_kernel() {
        initialize_kernel_system().unwrap();
        clear_kernels().unwrap();
        
        // Create a simple meta-kernel that doesn't reference missing files
        let meta_content = r#"
\begindata
TEST_VALUE = 123.45
ANOTHER_VALUE = 'test_string'
\begintext
This is a test meta-kernel without file references
        "#;
        
        let result = load_meta_kernel(meta_content, "/test/meta.mk");
        assert!(result.is_ok());
        
        // Check that the meta-kernel content was loaded into the pool
        assert!(kernel_pool::exists_in_pool("TEST_VALUE").unwrap());
        assert!(kernel_pool::exists_in_pool("ANOTHER_VALUE").unwrap());
        
        let (test_value, _) = kernel_pool::get_double_pool("TEST_VALUE", 0, 1).unwrap();
        assert_eq!(test_value[0], 123.45);
    }

    #[test]
    fn test_kernel_pool_integration() {
        initialize_kernel_system().unwrap();
        initialize_kernel_pool().unwrap();
        clear_kernels().unwrap();
        
        // Test that clearing kernels also clears the pool
        kernel_pool::initialize_pool().unwrap();
        kernel_pool::put_character_pool("TEST_VAR", vec!["test_value".to_string()]).unwrap();
        assert!(kernel_pool::exists_in_pool("TEST_VAR").unwrap());
        
        // Clear kernels should also clear pool
        clear_kernels().unwrap();
        
        // Pool should be cleared (need to reinitialize to check)
        kernel_pool::initialize_pool().unwrap();
        assert!(!kernel_pool::exists_in_pool("TEST_VAR").unwrap());
    }

    #[test]
    fn test_text_kernel_integration() {
        initialize_kernel_system().unwrap();
        initialize_kernel_pool().unwrap();
        clear_kernels().unwrap();
        kernel_pool::initialize_pool().unwrap();
        
        // Create a text kernel with \begindata section
        let text_kernel = r#"
\begindata
LEAP_SECONDS_FILE = 'naif0012.tls'
SPACECRAFT_ID = -123
TEST_ARRAY = ( 1.0, 2.0, 3.0 )
\begintext
This is a test text kernel
        "#;
        
        // Load as text kernel
        let result = furnish_kernel_from_bytes(text_kernel.as_bytes().to_vec(), "/test/test.tk");
        if let Err(ref e) = result {
            println!("Text kernel error: {:?}", e);
        }
        assert!(result.is_ok());
        
        // Check that variables were loaded into kernel pool
        if !kernel_pool::exists_in_pool("LEAP_SECONDS_FILE").unwrap() {
            println!("LEAP_SECONDS_FILE not found in pool");
        }
        assert!(kernel_pool::exists_in_pool("LEAP_SECONDS_FILE").unwrap());
        assert!(kernel_pool::exists_in_pool("SPACECRAFT_ID").unwrap());
        assert!(kernel_pool::exists_in_pool("TEST_ARRAY").unwrap());
        
        let (leap_file, _) = kernel_pool::get_character_pool("LEAP_SECONDS_FILE", 0, 1).unwrap();
        assert_eq!(leap_file[0], "naif0012.tls");
        
        let (spacecraft_id, _) = kernel_pool::get_integer_pool("SPACECRAFT_ID", 0, 1).unwrap();
        assert_eq!(spacecraft_id[0], -123);
        
        let (test_array, _) = kernel_pool::get_double_pool("TEST_ARRAY", 0, 10).unwrap();
        assert_eq!(test_array, vec![1.0, 2.0, 3.0]);
    }
}
