//! File I/O system for RustSPICE - WASM-compatible virtual file system
//! 
//! This module implements a virtual file system that can handle SPICE kernel files
//! without requiring actual file system access, making it compatible with WebAssembly.
//! 
//! The system supports:
//! - Loading kernel data from byte arrays (ArrayBuffer in JavaScript)
//! - Virtual file paths and handles 
//! - DAF (Double precision Array File) format support
//! - DAS (Direct Access, Segregated) format support
//! 
//! # CSPICE Equivalents
//! - Virtual file system replaces direct file I/O
//! - Handles kernel loading without filesystem access
//! - Maintains compatibility with CSPICE data formats

use crate::error_handling::{SpiceResult, SpiceError, SpiceErrorType};
use crate::foundation::{SpiceDouble, SpiceInt};
use std::collections::BTreeMap;
use std::vec::Vec;
use std::string::String;
use core::fmt;

/// Maximum number of simultaneously loaded kernels
pub const MAX_KERNELS: usize = 1000;

/// Maximum kernel file path length
pub const MAX_KERNEL_PATH: usize = 1024;

/// File handle type for loaded kernels
pub type FileHandle = i32;

/// Kernel file types as recognized by SPICE
#[derive(Debug, Clone, PartialEq)]
pub enum KernelType {
    /// SPK - Spacecraft and Planet Ephemeris Kernel (binary)
    SPK,
    /// CK - C-matrix (pointing) Kernel (binary) 
    CK,
    /// PCK - Planetary Constants Kernel (text or binary)
    PCK,
    /// EK - Events Kernel (binary)
    EK,
    /// IK - Instrument Kernel (text)
    IK,
    /// FK - Frame Kernel (text)
    FK,
    /// LSK - Leapseconds Kernel (text)
    LSK,
    /// SCLK - Spacecraft Clock Kernel (text)
    SCLK,
    /// MK - Meta-kernel (text)
    MK,
    /// Generic text kernel (IK, FK, LSK, SCLK, MK, etc.)
    TextKernel,
    /// Binary PCK format
    BinaryPCK,
    /// Unknown/unsupported type
    Unknown,
}

impl fmt::Display for KernelType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_str = match self {
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
            KernelType::Unknown => "Unknown",
        };
        write!(f, "{}", type_str)
    }
}

/// File architecture for SPICE kernels
#[derive(Debug, Clone, PartialEq)]
pub enum FileArchitecture {
    /// DAF - Double precision Array File
    DAF,
    /// DAS - Direct Access, Segregated
    DAS,
    /// Text file (not binary)
    Text,
    /// Transfer format
    XFR,
    /// Unknown architecture
    Unknown,
}

/// Represents a loaded kernel file in the virtual file system
#[derive(Debug, Clone)]
pub struct VirtualFile {
    /// File path/name
    pub path: String,
    /// Raw file data
    pub data: Vec<u8>,
    /// File architecture 
    pub architecture: FileArchitecture,
    /// Kernel type
    pub kernel_type: KernelType,
    /// File handle
    pub handle: FileHandle,
    /// Whether file is currently loaded
    pub loaded: bool,
}

impl VirtualFile {
    /// Create a new virtual file
    pub fn new(path: String, data: Vec<u8>, handle: FileHandle) -> Self {
        let (architecture, kernel_type) = detect_file_type(&data);
        
        VirtualFile {
            path,
            data,
            architecture,
            kernel_type,
            handle,
            loaded: false,
        }
    }
    
    /// Get file size in bytes
    pub fn size(&self) -> usize {
        self.data.len()
    }
    
    /// Check if file is binary
    pub fn is_binary(&self) -> bool {
        matches!(self.architecture, FileArchitecture::DAF | FileArchitecture::DAS)
    }
    
    /// Check if file is text
    pub fn is_text(&self) -> bool {
        matches!(self.architecture, FileArchitecture::Text)
    }
}

/// Virtual file system for WASM-compatible kernel management
#[derive(Debug)]
pub struct VirtualFileSystem {
    /// Map of file paths to virtual files
    files: BTreeMap<String, VirtualFile>,
    /// Map of handles to file paths
    handles: BTreeMap<FileHandle, String>,
    /// Next available file handle
    next_handle: FileHandle,
    /// List of currently loaded kernel paths in load order
    loaded_kernels: Vec<String>,
}

impl VirtualFileSystem {
    /// Create a new virtual file system
    pub fn new() -> Self {
        VirtualFileSystem {
            files: BTreeMap::new(),
            handles: BTreeMap::new(),
            next_handle: 1,
            loaded_kernels: Vec::new(),
        }
    }
    
    /// Load a kernel from raw bytes (equivalent to furnsh_c)
    /// This is the main entry point for loading kernels in WASM
    pub fn load_kernel_from_bytes(&mut self, data: Vec<u8>, path: &str) -> SpiceResult<FileHandle> {
        // Check if already loaded
        if self.files.contains_key(path) {
            return Err(SpiceError::new(
                SpiceErrorType::KernelAlreadyLoaded,
                format!("Kernel '{}' is already loaded", path),
            ));
        }
        
        // Check file limit
        if self.files.len() >= MAX_KERNELS {
            return Err(SpiceError::new(
                SpiceErrorType::TooManyKernels,
                format!("Maximum number of kernels ({}) exceeded", MAX_KERNELS),
            ));
        }
        
        // Validate path length
        if path.len() > MAX_KERNEL_PATH {
            return Err(SpiceError::new(
                SpiceErrorType::InvalidKernelPath,
                format!("Kernel path too long (max {} characters)", MAX_KERNEL_PATH),
            ));
        }
        
        // Validate data
        if data.is_empty() {
            return Err(SpiceError::new(
                SpiceErrorType::InvalidKernelData,
                "Kernel data cannot be empty".into(),
            ));
        }
        
        // Allocate handle
        let handle = self.next_handle;
        self.next_handle += 1;
        
        // Create virtual file
        let mut vfile = VirtualFile::new(path.to_string(), data, handle);
        
        // Validate file format
        if matches!(vfile.architecture, FileArchitecture::Unknown) {
            return Err(SpiceError::new(
                SpiceErrorType::InvalidKernelData,
                format!("Unrecognized file format for kernel '{}'", path),
            ));
        }
        
        // Mark as loaded
        vfile.loaded = true;
        
        // Store in maps
        self.handles.insert(handle, path.to_string());
        self.files.insert(path.to_string(), vfile);
        self.loaded_kernels.push(path.to_string());
        
        Ok(handle)
    }
    
    /// Unload a kernel by path (equivalent to unload_c)
    pub fn unload_kernel(&mut self, path: &str) -> SpiceResult<()> {
        // Find and remove the file
        if let Some(mut vfile) = self.files.remove(path) {
            vfile.loaded = false;
            
            // Remove handle mapping
            self.handles.remove(&vfile.handle);
            
            // Remove from loaded list
            self.loaded_kernels.retain(|p| p != path);
            
            Ok(())
        } else {
            Err(SpiceError::new(
                SpiceErrorType::KernelNotFound,
                format!("Kernel '{}' is not loaded", path),
            ))
        }
    }
    
    /// Clear all loaded kernels (equivalent to kclear_c)
    pub fn clear_all_kernels(&mut self) {
        self.files.clear();
        self.handles.clear();
        self.loaded_kernels.clear();
        self.next_handle = 1;
    }
    
    /// Get file by path
    pub fn get_file(&self, path: &str) -> Option<&VirtualFile> {
        self.files.get(path)
    }
    
    /// Get file by handle
    pub fn get_file_by_handle(&self, handle: FileHandle) -> Option<&VirtualFile> {
        if let Some(path) = self.handles.get(&handle) {
            self.files.get(path)
        } else {
            None
        }
    }
    
    /// Get list of loaded kernel paths
    pub fn loaded_kernel_paths(&self) -> &[String] {
        &self.loaded_kernels
    }
    
    /// Get number of loaded kernels
    pub fn kernel_count(&self) -> usize {
        self.loaded_kernels.len()
    }
    
    /// Check if a kernel is loaded
    pub fn is_kernel_loaded(&self, path: &str) -> bool {
        self.files.get(path).map_or(false, |f| f.loaded)
    }
    
    /// Get the list of loaded kernel paths
    pub fn list_loaded_kernels(&self) -> Vec<String> {
        self.files.values()
            .filter(|file| file.loaded)
            .map(|file| file.path.clone())
            .collect()
    }

    /// Get kernel data by path
    pub fn get_kernel_data(&self, path: &str) -> SpiceResult<Vec<u8>> {
        match self.files.get(path) {
            Some(file) => Ok(file.data.clone()),
            None => Err(SpiceError::new(
                SpiceErrorType::KernelNotFound,
                format!("Kernel '{}' not found", path),
            )),
        }
    }

    /// Get kernel info by path
    pub fn get_kernel_info(&self, path: &str) -> SpiceResult<KernelInfo> {
        self.kernel_info(path)
    }
    pub fn kernel_info(&self, path: &str) -> SpiceResult<KernelInfo> {
        if let Some(vfile) = self.files.get(path) {
            Ok(KernelInfo {
                path: vfile.path.clone(),
                kernel_type: vfile.kernel_type.clone(),
                architecture: vfile.architecture.clone(),
                size_bytes: vfile.size(),
                handle: vfile.handle,
                loaded: vfile.loaded,
            })
        } else {
            Err(SpiceError::new(
                SpiceErrorType::KernelNotFound,
                format!("Kernel '{}' not found", path),
            ))
        }
    }
}

impl Default for VirtualFileSystem {
    fn default() -> Self {
        Self::new()
    }
}

/// Information about a kernel file
#[derive(Debug, Clone)]
pub struct KernelInfo {
    /// File path
    pub path: String,
    /// Kernel type
    pub kernel_type: KernelType,
    /// File architecture
    pub architecture: FileArchitecture,
    /// Size in bytes
    pub size_bytes: usize,
    /// File handle
    pub handle: FileHandle,
    /// Whether currently loaded
    pub loaded: bool,
}

/// Detect file type and architecture from file header
/// This examines the first few bytes to determine DAF/DAS/Text format
fn detect_file_type(data: &[u8]) -> (FileArchitecture, KernelType) {
    if data.len() < 8 {
        return (FileArchitecture::Unknown, KernelType::Unknown);
    }
    
    // Check for DAF file signature
    if data.len() >= 8 && &data[0..8] == b"DAF/SPK " {
        return (FileArchitecture::DAF, KernelType::SPK);
    }
    if data.len() >= 8 && &data[0..8] == b"DAF/CK  " {
        return (FileArchitecture::DAF, KernelType::CK);
    }
    if data.len() >= 8 && &data[0..8] == b"DAF/PCK " {
        return (FileArchitecture::DAF, KernelType::PCK);
    }
    
    // Check for DAS file signature
    if data.len() >= 8 && &data[0..8] == b"DAS/EK  " {
        return (FileArchitecture::DAS, KernelType::EK);
    }
    if data.len() >= 8 && &data[0..8] == b"DAS/    " {
        return (FileArchitecture::DAS, KernelType::Unknown);
    }
    
    // Check for text kernels by examining content
    if let Ok(text) = core::str::from_utf8(data) {
        // Look for common text kernel patterns
        if text.contains("\\begindata") || text.contains("\\begintext") ||
           text.contains("BODY") || text.contains("DELTET") ||
           text.contains("LEAP_SECONDS") || text.contains("KERNELS_TO_LOAD") ||
           text.contains("FRAME_") || text.contains("INS") || text.contains("SCLK") {
            
            // Detect specific text kernel types
            if text.contains("LEAP_SECONDS") || text.contains("DELTET") {
                return (FileArchitecture::Text, KernelType::LSK);
            }
            if text.contains("BODY") && text.contains("RADII") {
                return (FileArchitecture::Text, KernelType::PCK);
            }
            if text.contains("FRAME_") {
                return (FileArchitecture::Text, KernelType::FK);
            }
            if text.contains("INS") && text.contains("INSTRUMENT") {
                return (FileArchitecture::Text, KernelType::IK);
            }
            if text.contains("SCLK") {
                return (FileArchitecture::Text, KernelType::SCLK);
            }
            if text.contains("KERNELS_TO_LOAD") {
                return (FileArchitecture::Text, KernelType::MK);
            }
            // Generic text kernel
            return (FileArchitecture::Text, KernelType::PCK);
        }
    }
    
    // Check for transfer format
    if data.len() >= 8 && &data[0..8] == b"'NAIF/XF" {
        return (FileArchitecture::XFR, KernelType::Unknown);
    }
    
    (FileArchitecture::Unknown, KernelType::Unknown)
}

/// Read bytes from a virtual file at a specific offset
pub fn read_file_bytes(vfs: &VirtualFileSystem, path: &str, offset: usize, length: usize) -> SpiceResult<Vec<u8>> {
    if let Some(vfile) = vfs.get_file(path) {
        if offset >= vfile.data.len() {
            return Err(SpiceError::new(
                SpiceErrorType::FileReadError,
                format!("Offset {} beyond file size {}", offset, vfile.data.len()),
            ));
        }
        
        let end = core::cmp::min(offset + length, vfile.data.len());
        Ok(vfile.data[offset..end].to_vec())
    } else {
        Err(SpiceError::new(
            SpiceErrorType::KernelNotFound,
            format!("File '{}' not found", path),
        ))
    }
}

/// Read a range of double precision numbers from a binary file
pub fn read_doubles(vfs: &VirtualFileSystem, path: &str, start_address: usize, count: usize) -> SpiceResult<Vec<SpiceDouble>> {
    let offset = start_address * 8; // 8 bytes per double
    let byte_length = count * 8;
    
    let bytes = read_file_bytes(vfs, path, offset, byte_length)?;
    
    if bytes.len() < byte_length {
        return Err(SpiceError::new(
            SpiceErrorType::FileReadError,
            format!("Insufficient data: requested {} bytes, got {}", byte_length, bytes.len()),
        ));
    }
    
    let mut doubles = Vec::with_capacity(count);
    for i in 0..count {
        let byte_offset = i * 8;
        if byte_offset + 8 <= bytes.len() {
            let double_bytes = &bytes[byte_offset..byte_offset + 8];
            let value = f64::from_le_bytes([
                double_bytes[0], double_bytes[1], double_bytes[2], double_bytes[3],
                double_bytes[4], double_bytes[5], double_bytes[6], double_bytes[7],
            ]);
            doubles.push(value);
        }
    }
    
    Ok(doubles)
}

/// Read a range of integers from a binary file  
pub fn read_integers(vfs: &VirtualFileSystem, path: &str, start_address: usize, count: usize) -> SpiceResult<Vec<SpiceInt>> {
    let offset = start_address * 4; // 4 bytes per int
    let byte_length = count * 4;
    
    let bytes = read_file_bytes(vfs, path, offset, byte_length)?;
    
    if bytes.len() < byte_length {
        return Err(SpiceError::new(
            SpiceErrorType::FileReadError,
            format!("Insufficient data: requested {} bytes, got {}", byte_length, bytes.len()),
        ));
    }
    
    let mut integers = Vec::with_capacity(count);
    for i in 0..count {
        let byte_offset = i * 4;
        if byte_offset + 4 <= bytes.len() {
            let int_bytes = &bytes[byte_offset..byte_offset + 4];
            let value = i32::from_le_bytes([
                int_bytes[0], int_bytes[1], int_bytes[2], int_bytes[3],
            ]);
            integers.push(value);
        }
    }
    
    Ok(integers)
}

/// Legacy function for compatibility - checks if a file is loaded in VFS
pub fn file_exists(_filename: &str) -> bool {
    // For now just return false as this would need access to global VFS instance
    // In practice, kernels should be loaded via the kernel_system module
    false
}

/// Legacy function for compatibility - reads file data if loaded
pub fn read_file(_filename: &str) -> SpiceResult<Vec<u8>> {
    // For now return error as this would need access to global VFS instance  
    // In practice, file data should be accessed via VFS instance
    Err(SpiceError::new(
        SpiceErrorType::FileIOError,
        "Use VirtualFileSystem for file operations in WASM environment".into()
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_virtual_file_system_creation() {
        let mut vfs = VirtualFileSystem::new();
        assert_eq!(vfs.kernel_count(), 0);
        assert!(vfs.list_loaded_kernels().is_empty());
    }

    #[test]
    fn test_kernel_type_detection() {
        // Test SPK/DAF kernel detection
        let spk_data = b"DAF/SPK      test";
        assert_eq!(detect_kernel_type(spk_data), KernelType::SPK);
        
        // Test CK/DAF kernel detection
        let ck_data = b"DAF/CK       test";
        assert_eq!(detect_kernel_type(ck_data), KernelType::CK);
        
        // Test PCK text kernel detection
        let pck_data = b"\nBODY399_RADII = ( 6378.1366 6378.1366 6356.7519 )\n";
        assert_eq!(detect_kernel_type(pck_data), KernelType::PCK);
        
        // Test LSK kernel detection
        let lsk_data = b"\nDELTET/DELTA_T_A = 32.184\n";
        assert_eq!(detect_kernel_type(lsk_data), KernelType::LSK);
        
        // Test EK/DAS kernel detection
        let ek_data = b"DAS/EK       test";
        assert_eq!(detect_kernel_type(ek_data), KernelType::EK);
        
        // Test binary PCK kernel detection
        let binary_pck_data = b"DAF/PCK      test";
        assert_eq!(detect_kernel_type(binary_pck_data), KernelType::PCK);
        
        // Test unknown kernel detection
        let unknown_data = b"some random data";
        assert_eq!(detect_kernel_type(unknown_data), KernelType::Unknown);
    }

    #[test]
    fn test_file_architecture_detection() {
        // Test DAF architecture
        let daf_data = b"DAF/SPK      test";
        assert_eq!(detect_file_architecture(daf_data), FileArchitecture::DAF);
        
        // Test DAS architecture
        let das_data = b"DAS/EK       test";
        assert_eq!(detect_file_architecture(das_data), FileArchitecture::DAS);
        
        // Test text file
        let text_data = b"This is a text kernel\nKERNELS_TO_LOAD = ( )";
        assert_eq!(detect_file_architecture(text_data), FileArchitecture::Text);
        
        // Test unknown
        let unknown_data = b"random binary data";
        assert_eq!(detect_file_architecture(unknown_data), FileArchitecture::Unknown);
    }

    #[test]
    fn test_load_kernel_from_bytes() {
        let mut vfs = VirtualFileSystem::new();
        
        let test_data = b"DAF/SPK      test kernel data".to_vec();
        let path = "/test/kernel.bsp";
        
        let handle = vfs.load_kernel_from_bytes(test_data, path).unwrap();
        assert_eq!(handle, 1);
        assert_eq!(vfs.kernel_count(), 1);
        assert!(vfs.is_kernel_loaded(path));
        
        let loaded_kernels = vfs.list_loaded_kernels();
        assert_eq!(loaded_kernels.len(), 1);
        assert_eq!(loaded_kernels[0], path);
    }

    #[test]
    fn test_load_multiple_kernels() {
        let mut vfs = VirtualFileSystem::new();
        
        // Load SPK kernel
        let spk_data = b"DAF/SPK      spk test".to_vec();
        let spk_handle = vfs.load_kernel_from_bytes(spk_data, "/test/planets.bsp").unwrap();
        
        // Load CK kernel
        let ck_data = b"DAF/CK       ck test".to_vec();
        let ck_handle = vfs.load_kernel_from_bytes(ck_data, "/test/attitude.bc").unwrap();
        
        assert_eq!(spk_handle, 1);
        assert_eq!(ck_handle, 2);
        assert_eq!(vfs.kernel_count(), 2);
        
        assert!(vfs.is_kernel_loaded("/test/planets.bsp"));
        assert!(vfs.is_kernel_loaded("/test/attitude.bc"));
        
        let loaded_kernels = vfs.list_loaded_kernels();
        assert_eq!(loaded_kernels.len(), 2);
        assert!(loaded_kernels.contains(&"/test/planets.bsp".to_string()));
        assert!(loaded_kernels.contains(&"/test/attitude.bc".to_string()));
    }

    #[test]
    fn test_duplicate_kernel_loading() {
        let mut vfs = VirtualFileSystem::new();
        
        let test_data = b"DAF/SPK      test".to_vec();
        let path = "/test/kernel.bsp";
        
        // Load first time
        let handle1 = vfs.load_kernel_from_bytes(test_data.clone(), path).unwrap();
        assert_eq!(handle1, 1);
        
        // Try to load same path again - should return error
        let result = vfs.load_kernel_from_bytes(test_data, path);
        assert!(result.is_err());
        
        if let Err(err) = result {
            assert_eq!(err.error_type, SpiceErrorType::KernelAlreadyLoaded);
        }
        
        assert_eq!(vfs.kernel_count(), 1); // Should still be 1
    }

    #[test]
    fn test_unload_kernel() {
        let mut vfs = VirtualFileSystem::new();
        
        let test_data = b"DAF/SPK      test".to_vec();
        let path = "/test/kernel.bsp";
        
        vfs.load_kernel_from_bytes(test_data, path).unwrap();
        assert_eq!(vfs.kernel_count(), 1);
        assert!(vfs.is_kernel_loaded(path));
        
        vfs.unload_kernel(path).unwrap();
        assert_eq!(vfs.kernel_count(), 0);
        assert!(!vfs.is_kernel_loaded(path));
        
        let loaded_kernels = vfs.list_loaded_kernels();
        assert!(loaded_kernels.is_empty());
    }

    #[test]
    fn test_unload_nonexistent_kernel() {
        let mut vfs = VirtualFileSystem::new();
        
        let result = vfs.unload_kernel("/nonexistent/kernel.bsp");
        assert!(result.is_err());
        
        if let Err(err) = result {
            assert_eq!(err.error_type, SpiceErrorType::KernelNotFound);
        }
    }

    #[test]
    fn test_clear_all_kernels() {
        let mut vfs = VirtualFileSystem::new();
        
        // Load multiple kernels
        vfs.load_kernel_from_bytes(b"DAF/SPK test1".to_vec(), "/test/kernel1.bsp").unwrap();
        vfs.load_kernel_from_bytes(b"DAF/CK  test2".to_vec(), "/test/kernel2.bc").unwrap();
        vfs.load_kernel_from_bytes(b"DAS/EK  test3".to_vec(), "/test/kernel3.ek").unwrap();
        
        assert_eq!(vfs.kernel_count(), 3);
        
        vfs.clear_all_kernels();
        assert_eq!(vfs.kernel_count(), 0);
        assert!(vfs.list_loaded_kernels().is_empty());
    }

    #[test]
    fn test_get_kernel_data() {
        let mut vfs = VirtualFileSystem::new();
        
        let test_data = b"DAF/SPK      test kernel data".to_vec();
        let path = "/test/kernel.bsp";
        
        vfs.load_kernel_from_bytes(test_data.clone(), path).unwrap();
        
        let retrieved_data = vfs.get_kernel_data(path).unwrap();
        assert_eq!(retrieved_data, test_data);
    }

    #[test]
    fn test_get_kernel_info() {
        let mut vfs = VirtualFileSystem::new();
        
        let test_data = b"DAF/CK       test ck kernel".to_vec();
        let path = "/test/attitude.bc";
        
        vfs.load_kernel_from_bytes(test_data, path).unwrap();
        
        let info = vfs.get_kernel_info(path).unwrap();
        assert_eq!(info.path, path);
        assert_eq!(info.kernel_type, KernelType::CK);
        assert_eq!(info.architecture, FileArchitecture::DAF);
        assert!(info.loaded);
        assert!(info.size_bytes > 0);
    }

    #[test]
    fn test_max_kernels_limit() {
        let mut vfs = VirtualFileSystem::new();
        
        // Try to exceed MAX_KERNELS limit
        // We'll simulate this by checking that we get an error when trying to load too many
        for i in 0..MAX_KERNELS {
            let data = format!("DAF/SPK      test{}", i).into_bytes();
            let path = format!("/test/kernel{}.bsp", i);
            let result = vfs.load_kernel_from_bytes(data, &path);
            assert!(result.is_ok(), "Failed to load kernel {}", i);
        }
        
        // Now try to load one more - should fail
        let extra_data = b"DAF/SPK      extra".to_vec();
        let extra_path = "/test/extra.bsp";
        let result = vfs.load_kernel_from_bytes(extra_data, extra_path);
        assert!(result.is_err());
        
        if let Err(err) = result {
            assert_eq!(err.error_type, SpiceErrorType::TooManyKernels);
        }
    }

    #[test]
    fn test_read_doubles_from_data() {
        // Create test data with some doubles in little-endian format
        let mut data = Vec::new();
        let test_doubles = [1.0_f64, 2.5_f64, -3.14159_f64];
        
        for &val in &test_doubles {
            data.extend_from_slice(&val.to_le_bytes());
        }
        
        let result = read_doubles_from_data(&data, 0, 3).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], 1.0);
        assert_eq!(result[1], 2.5);
        assert!((result[2] + 3.14159).abs() < 1e-10);
    }

    #[test]
    fn test_read_integers_from_data() {
        // Create test data with some integers in little-endian format
        let mut data = Vec::new();
        let test_ints = [42_i32, -100_i32, 2147483647_i32];
        
        for &val in &test_ints {
            data.extend_from_slice(&val.to_le_bytes());
        }
        
        let result = read_integers_from_data(&data, 0, 3).unwrap();
        assert_eq!(result.len(), 3);
        assert_eq!(result[0], 42);
        assert_eq!(result[1], -100);
        assert_eq!(result[2], 2147483647);
    }

    #[test]
    fn test_read_data_bounds_checking() {
        let data = vec![1, 2, 3, 4, 5, 6, 7, 8];
        
        // Test reading beyond data bounds
        let result = read_doubles_from_data(&data, 0, 2); // Need 16 bytes, only have 8
        assert!(result.is_err());
        
        let result = read_integers_from_data(&data, 0, 3); // Need 12 bytes, only have 8
        assert!(result.is_err());
        
        // Test reading with invalid offset
        let result = read_doubles_from_data(&data, 10, 1);
        assert!(result.is_err());
    }

    #[test]
    fn test_file_exists_stub() {
        // Legacy compatibility test
        assert!(!file_exists("nonexistent.txt"));
    }

    #[test]
    fn test_read_file_stub() {
        // Legacy compatibility test
        let result = read_file("nonexistent.txt");
        assert!(result.is_err());
        match result {
            Err(SpiceError { error_type: SpiceErrorType::FileIOError, .. }) => (),
            _ => panic!("Expected FileIOError"),
        }
    }
}

/// Detect kernel type from data (wrapper for test compatibility)
pub fn detect_kernel_type(data: &[u8]) -> KernelType {
    let (_, kernel_type) = detect_file_type(data);
    kernel_type
}

/// Detect file architecture from data (wrapper for test compatibility)  
pub fn detect_file_architecture(data: &[u8]) -> FileArchitecture {
    let (architecture, _) = detect_file_type(data);
    architecture
}

/// Read doubles from data at specific offset (wrapper for test compatibility)
pub fn read_doubles_from_data(data: &[u8], offset: usize, count: usize) -> SpiceResult<Vec<SpiceDouble>> {
    let byte_length = count * 8; // 8 bytes per double
    
    if offset + byte_length > data.len() {
        return Err(SpiceError::new(
            SpiceErrorType::InvalidKernelData,
            format!("Attempt to read {} bytes at offset {} from data of length {}", 
                    byte_length, offset, data.len()),
        ));
    }
    
    let mut doubles = Vec::new();
    for i in 0..count {
        let byte_start = offset + i * 8;
        let double_bytes = &data[byte_start..byte_start + 8];
        let value = f64::from_le_bytes(double_bytes.try_into().unwrap());
        doubles.push(value);
    }
    
    Ok(doubles)
}

/// Read integers from data at specific offset (wrapper for test compatibility)
pub fn read_integers_from_data(data: &[u8], offset: usize, count: usize) -> SpiceResult<Vec<SpiceInt>> {
    let byte_length = count * 4; // 4 bytes per int
    
    if offset + byte_length > data.len() {
        return Err(SpiceError::new(
            SpiceErrorType::InvalidKernelData,
            format!("Attempt to read {} bytes at offset {} from data of length {}", 
                    byte_length, offset, data.len()),
        ));
    }
    
    let mut integers = Vec::new();
    for i in 0..count {
        let byte_start = offset + i * 4;
        let int_bytes = &data[byte_start..byte_start + 4];
        let value = i32::from_le_bytes(int_bytes.try_into().unwrap());
        integers.push(value);
    }
    
    Ok(integers)
}
