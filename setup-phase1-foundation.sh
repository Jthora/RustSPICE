#!/bin/bash

# Phase 1 Implementation Setup
# Creates the foundational Rust structure for CSPICE conversion

set -e

echo "🚀 Setting up RustSPICE Phase 1: Foundation"
echo "==========================================="

# Clean up previous WASM placeholder and create new structure
echo "📁 Creating new Rust project structure..."

# Create the core library structure
mkdir -p src/foundation
mkdir -p src/time_system  
mkdir -p src/coordinates
mkdir -p src/ephemeris
mkdir -p src/file_system
mkdir -p src/math_core
mkdir -p src/error_handling
mkdir -p src/kernel_system
mkdir -p src/body_data

# Create new lib.rs for actual CSPICE conversion
cat > src/lib.rs << 'EOF'
//! RustSPICE: Complete Rust implementation of NASA's SPICE toolkit
//! 
//! This is a from-scratch conversion of the entire CSPICE library to Rust,
//! designed for WebAssembly compatibility while maintaining full numerical
//! accuracy and functional equivalence with the original CSPICE.

#![no_std]
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use alloc::collections::BTreeMap;

// Core modules
pub mod error_handling;
pub mod math_core;
pub mod foundation;
pub mod time_system;
pub mod coordinates;
pub mod file_system;
pub mod kernel_system;
pub mod ephemeris;
pub mod body_data;

// Re-export the most important types and functions
pub use error_handling::{SpiceError, SpiceResult, ErrorTrace};
pub use foundation::{
    SpiceDouble, SpiceInt, SpiceChar, SpiceBoolean,
    SpiceMatrix3x3, SpiceMatrix6x6, SpiceVector3, SpiceVector6,
    StateVector, EphemerisTime, JulianDate
};
pub use time_system::{str_to_et, et_to_utc, utc_to_et, time_output};
pub use ephemeris::{ephemeris_state, ephemeris_position};
pub use kernel_system::{furnish_kernel, unload_kernel, clear_kernels};

// WASM bindings - only when targeting wasm32
#[cfg(target_arch = "wasm32")]
pub mod wasm_bindings;

#[cfg(target_arch = "wasm32")]
pub use wasm_bindings::*;

/// Initialize the RustSPICE library
/// This sets up global state and error handling
pub fn initialize() -> SpiceResult<()> {
    error_handling::initialize_error_system()?;
    kernel_system::initialize_kernel_system()?;
    Ok(())
}

/// Get the version of RustSPICE
pub fn version() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

/// Get build information
pub fn build_info() -> String {
    format!("RustSPICE v{} - Complete CSPICE conversion for WASM", version())
}
EOF

# Create error handling module (highest priority based on dependency analysis)
cat > src/error_handling.rs << 'EOF'
//! Error handling system for RustSPICE
//! 
//! This module replaces CSPICE's error handling system (chkin_/chkout_/sigerr_)
//! with Rust's native Result type and structured error handling.

use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use core::fmt;

/// SPICE error types corresponding to different failure modes
#[derive(Debug, Clone, PartialEq)]
pub enum SpiceErrorType {
    /// Kernel file not found or could not be loaded
    KernelNotFound,
    /// Invalid time string or time conversion error
    InvalidTime,
    /// Invalid target body name or ID
    InvalidTarget,
    /// Invalid observer body name or ID  
    InvalidObserver,
    /// Invalid reference frame name
    InvalidFrame,
    /// Invalid aberration correction specification
    InvalidAberrationCorrection,
    /// Mathematical computation error (division by zero, etc.)
    ComputationError,
    /// File I/O error
    FileIOError,
    /// Memory allocation error
    MemoryError,
    /// Invalid function argument
    InvalidArgument,
    /// Kernel data insufficient for requested operation
    InsufficientData,
    /// Numerical integration or interpolation failure
    NumericalError,
    /// Generic SPICE error
    SpiceError,
}

/// Structured error with type, message, and optional call trace
#[derive(Debug, Clone)]
pub struct SpiceError {
    pub error_type: SpiceErrorType,
    pub message: String,
    pub function_trace: Vec<String>,
    pub details: Option<String>,
}

impl SpiceError {
    /// Create a new SpiceError
    pub fn new(error_type: SpiceErrorType, message: String) -> Self {
        SpiceError {
            error_type,
            message,
            function_trace: Vec::new(),
            details: None,
        }
    }

    /// Add function to the call trace (equivalent to chkin_ in CSPICE)
    pub fn add_trace(mut self, function_name: String) -> Self {
        self.function_trace.push(function_name);
        self
    }

    /// Add additional details to the error
    pub fn with_details(mut self, details: String) -> Self {
        self.details = Some(details);
        self
    }

    /// Get the error type as a string
    pub fn error_type_string(&self) -> &'static str {
        match self.error_type {
            SpiceErrorType::KernelNotFound => "KERNEL_NOT_FOUND",
            SpiceErrorType::InvalidTime => "INVALID_TIME",
            SpiceErrorType::InvalidTarget => "INVALID_TARGET",
            SpiceErrorType::InvalidObserver => "INVALID_OBSERVER",
            SpiceErrorType::InvalidFrame => "INVALID_FRAME",
            SpiceErrorType::InvalidAberrationCorrection => "INVALID_ABERRATION_CORRECTION",
            SpiceErrorType::ComputationError => "COMPUTATION_ERROR",
            SpiceErrorType::FileIOError => "FILE_IO_ERROR",
            SpiceErrorType::MemoryError => "MEMORY_ERROR",
            SpiceErrorType::InvalidArgument => "INVALID_ARGUMENT",
            SpiceErrorType::InsufficientData => "INSUFFICIENT_DATA",
            SpiceErrorType::NumericalError => "NUMERICAL_ERROR",
            SpiceErrorType::SpiceError => "SPICE_ERROR",
        }
    }
}

impl fmt::Display for SpiceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.error_type_string(), self.message)?;
        
        if !self.function_trace.is_empty() {
            write!(f, "\nCall trace:")?;
            for (i, func) in self.function_trace.iter().enumerate() {
                write!(f, "\n  {}: {}", i + 1, func)?;
            }
        }
        
        if let Some(details) = &self.details {
            write!(f, "\nDetails: {}", details)?;
        }
        
        Ok(())
    }
}

/// Result type for SPICE operations
pub type SpiceResult<T> = Result<T, SpiceError>;

/// Error trace for debugging - equivalent to CSPICE call stack
#[derive(Debug, Clone)]
pub struct ErrorTrace {
    function_stack: Vec<String>,
}

impl ErrorTrace {
    pub fn new() -> Self {
        ErrorTrace {
            function_stack: Vec::new(),
        }
    }

    /// Enter a function (equivalent to chkin_)
    pub fn enter_function(&mut self, name: String) {
        self.function_stack.push(name);
    }

    /// Exit a function (equivalent to chkout_)
    pub fn exit_function(&mut self) {
        self.function_stack.pop();
    }

    /// Get current function stack
    pub fn get_stack(&self) -> &[String] {
        &self.function_stack
    }
}

/// Global error trace for debugging
static mut GLOBAL_ERROR_TRACE: Option<ErrorTrace> = None;

/// Initialize the error handling system
pub fn initialize_error_system() -> SpiceResult<()> {
    unsafe {
        GLOBAL_ERROR_TRACE = Some(ErrorTrace::new());
    }
    Ok(())
}

/// Check if the error system has failed (equivalent to failed_)
pub fn has_failed() -> bool {
    // In Rust, we use Result types instead of global error state
    // This function is provided for compatibility but should rarely be used
    false
}

/// Reset error state (equivalent to reset_)
pub fn reset_error_state() {
    // In Rust, each operation returns its own Result
    // This function is provided for compatibility but is largely unnecessary
}

/// Convenience macros for error creation

/// Create a SpiceError with automatic function tracing
#[macro_export]
macro_rules! spice_error {
    ($error_type:expr, $message:expr) => {
        SpiceError::new($error_type, $message.into()).add_trace(function_name!().into())
    };
    ($error_type:expr, $fmt:expr, $($arg:tt)*) => {
        SpiceError::new($error_type, format!($fmt, $($arg)*)).add_trace(function_name!().into())
    };
}

/// Return early with a SpiceError
#[macro_export]
macro_rules! spice_bail {
    ($error_type:expr, $message:expr) => {
        return Err(spice_error!($error_type, $message))
    };
    ($error_type:expr, $fmt:expr, $($arg:tt)*) => {
        return Err(spice_error!($error_type, $fmt, $($arg)*))
    };
}

/// Function name helper for error tracing
#[macro_export] 
macro_rules! function_name {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            core::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_creation() {
        let error = SpiceError::new(
            SpiceErrorType::InvalidTime,
            "Invalid time string".to_string()
        );
        assert_eq!(error.error_type, SpiceErrorType::InvalidTime);
        assert_eq!(error.message, "Invalid time string");
    }

    #[test]
    fn test_error_trace() {
        let error = SpiceError::new(
            SpiceErrorType::ComputationError,
            "Division by zero".to_string()
        ).add_trace("compute_position".to_string())
         .add_trace("ephemeris_state".to_string());
        
        assert_eq!(error.function_trace.len(), 2);
        assert_eq!(error.function_trace[0], "compute_position");
        assert_eq!(error.function_trace[1], "ephemeris_state");
    }
}
EOF

# Create foundation data types module
cat > src/foundation.rs << 'EOF'
//! Foundation data types for RustSPICE
//! 
//! This module defines the core data structures that correspond to SPICE
//! data types and provides the foundation for all other modules.

use alloc::string::String;
use alloc::vec::Vec;
use core::ops::{Add, Sub, Mul, Div, Index, IndexMut};
use crate::error_handling::{SpiceResult, SpiceError, SpiceErrorType};

/// SPICE double precision floating point (f64)
pub type SpiceDouble = f64;

/// SPICE integer (i32)
pub type SpiceInt = i32;

/// SPICE character string
pub type SpiceChar = String;

/// SPICE boolean
pub type SpiceBoolean = bool;

/// 3x3 matrix for rotation transformations
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpiceMatrix3x3(pub [[SpiceDouble; 3]; 3]);

impl SpiceMatrix3x3 {
    /// Create a new 3x3 matrix
    pub fn new(data: [[SpiceDouble; 3]; 3]) -> Self {
        SpiceMatrix3x3(data)
    }

    /// Create identity matrix
    pub fn identity() -> Self {
        SpiceMatrix3x3([
            [1.0, 0.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
        ])
    }

    /// Create zero matrix
    pub fn zeros() -> Self {
        SpiceMatrix3x3([[0.0; 3]; 3])
    }

    /// Matrix transpose
    pub fn transpose(&self) -> Self {
        let mut result = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                result[i][j] = self.0[j][i];
            }
        }
        SpiceMatrix3x3(result)
    }

    /// Matrix determinant
    pub fn determinant(&self) -> SpiceDouble {
        let m = &self.0;
        m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
            - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
            + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
    }
}

impl Index<usize> for SpiceMatrix3x3 {
    type Output = [SpiceDouble; 3];
    fn index(&self, index: usize) -> &Self::Output {
        &self.0[index]
    }
}

impl IndexMut<usize> for SpiceMatrix3x3 {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.0[index]
    }
}

/// 6x6 matrix for state transformations
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpiceMatrix6x6(pub [[SpiceDouble; 6]; 6]);

impl SpiceMatrix6x6 {
    pub fn new(data: [[SpiceDouble; 6]; 6]) -> Self {
        SpiceMatrix6x6(data)
    }

    pub fn identity() -> Self {
        let mut matrix = [[0.0; 6]; 6];
        for i in 0..6 {
            matrix[i][i] = 1.0;
        }
        SpiceMatrix6x6(matrix)
    }

    pub fn zeros() -> Self {
        SpiceMatrix6x6([[0.0; 6]; 6])
    }
}

/// 3D vector for positions and directions
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpiceVector3(pub [SpiceDouble; 3]);

impl SpiceVector3 {
    pub fn new(x: SpiceDouble, y: SpiceDouble, z: SpiceDouble) -> Self {
        SpiceVector3([x, y, z])
    }

    pub fn zeros() -> Self {
        SpiceVector3([0.0, 0.0, 0.0])
    }

    pub fn x(&self) -> SpiceDouble { self.0[0] }
    pub fn y(&self) -> SpiceDouble { self.0[1] }
    pub fn z(&self) -> SpiceDouble { self.0[2] }

    /// Vector magnitude (norm)
    pub fn magnitude(&self) -> SpiceDouble {
        (self.0[0] * self.0[0] + self.0[1] * self.0[1] + self.0[2] * self.0[2]).sqrt()
    }

    /// Unit vector (normalized)
    pub fn unit(&self) -> SpiceResult<SpiceVector3> {
        let mag = self.magnitude();
        if mag == 0.0 {
            return Err(SpiceError::new(
                SpiceErrorType::ComputationError,
                "Cannot normalize zero vector".to_string()
            ));
        }
        Ok(SpiceVector3([
            self.0[0] / mag,
            self.0[1] / mag,
            self.0[2] / mag,
        ]))
    }

    /// Dot product
    pub fn dot(&self, other: &SpiceVector3) -> SpiceDouble {
        self.0[0] * other.0[0] + self.0[1] * other.0[1] + self.0[2] * other.0[2]
    }

    /// Cross product
    pub fn cross(&self, other: &SpiceVector3) -> SpiceVector3 {
        SpiceVector3([
            self.0[1] * other.0[2] - self.0[2] * other.0[1],
            self.0[2] * other.0[0] - self.0[0] * other.0[2],
            self.0[0] * other.0[1] - self.0[1] * other.0[0],
        ])
    }
}

impl Add for SpiceVector3 {
    type Output = SpiceVector3;
    fn add(self, other: SpiceVector3) -> SpiceVector3 {
        SpiceVector3([
            self.0[0] + other.0[0],
            self.0[1] + other.0[1],
            self.0[2] + other.0[2],
        ])
    }
}

impl Sub for SpiceVector3 {
    type Output = SpiceVector3;
    fn sub(self, other: SpiceVector3) -> SpiceVector3 {
        SpiceVector3([
            self.0[0] - other.0[0],
            self.0[1] - other.0[1],
            self.0[2] - other.0[2],
        ])
    }
}

impl Mul<SpiceDouble> for SpiceVector3 {
    type Output = SpiceVector3;
    fn mul(self, scalar: SpiceDouble) -> SpiceVector3 {
        SpiceVector3([
            self.0[0] * scalar,
            self.0[1] * scalar,
            self.0[2] * scalar,
        ])
    }
}

/// 6D vector for state (position + velocity)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SpiceVector6(pub [SpiceDouble; 6]);

impl SpiceVector6 {
    pub fn new(data: [SpiceDouble; 6]) -> Self {
        SpiceVector6(data)
    }

    pub fn from_position_velocity(pos: SpiceVector3, vel: SpiceVector3) -> Self {
        SpiceVector6([
            pos.0[0], pos.0[1], pos.0[2],
            vel.0[0], vel.0[1], vel.0[2],
        ])
    }

    pub fn position(&self) -> SpiceVector3 {
        SpiceVector3([self.0[0], self.0[1], self.0[2]])
    }

    pub fn velocity(&self) -> SpiceVector3 {
        SpiceVector3([self.0[3], self.0[4], self.0[5]])
    }
}

/// State vector combining position and velocity with light time
#[derive(Debug, Clone, PartialEq)]
pub struct StateVector {
    pub position: SpiceVector3,
    pub velocity: SpiceVector3,
    pub light_time: SpiceDouble,
}

impl StateVector {
    pub fn new(
        position: SpiceVector3,
        velocity: SpiceVector3,
        light_time: SpiceDouble,
    ) -> Self {
        StateVector {
            position,
            velocity,
            light_time,
        }
    }

    /// Get position magnitude (distance)
    pub fn position_magnitude(&self) -> SpiceDouble {
        self.position.magnitude()
    }

    /// Get velocity magnitude (speed)
    pub fn velocity_magnitude(&self) -> SpiceDouble {
        self.velocity.magnitude()
    }

    /// Convert to 6D vector
    pub fn to_vector6(&self) -> SpiceVector6 {
        SpiceVector6::from_position_velocity(self.position, self.velocity)
    }
}

/// Ephemeris time (seconds past J2000 epoch)
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct EphemerisTime(pub SpiceDouble);

impl EphemerisTime {
    pub fn new(seconds_past_j2000: SpiceDouble) -> Self {
        EphemerisTime(seconds_past_j2000)
    }

    pub fn j2000() -> Self {
        EphemerisTime(0.0)
    }

    pub fn seconds(&self) -> SpiceDouble {
        self.0
    }

    pub fn add_seconds(&self, seconds: SpiceDouble) -> Self {
        EphemerisTime(self.0 + seconds)
    }
}

impl Add<SpiceDouble> for EphemerisTime {
    type Output = EphemerisTime;
    fn add(self, seconds: SpiceDouble) -> EphemerisTime {
        EphemerisTime(self.0 + seconds)
    }
}

impl Sub for EphemerisTime {
    type Output = SpiceDouble;
    fn sub(self, other: EphemerisTime) -> SpiceDouble {
        self.0 - other.0
    }
}

/// Julian date
#[derive(Debug, Clone, Copy, PartialEq, PartialOrd)]
pub struct JulianDate(pub SpiceDouble);

impl JulianDate {
    pub fn new(julian_day: SpiceDouble) -> Self {
        JulianDate(julian_day)
    }

    pub fn j2000() -> Self {
        JulianDate(2451545.0)
    }

    pub fn days(&self) -> SpiceDouble {
        self.0
    }

    /// Convert to ephemeris time
    pub fn to_ephemeris_time(&self) -> EphemerisTime {
        EphemerisTime((self.0 - 2451545.0) * 86400.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vector_operations() {
        let v1 = SpiceVector3::new(1.0, 2.0, 3.0);
        let v2 = SpiceVector3::new(4.0, 5.0, 6.0);

        let sum = v1 + v2;
        assert_eq!(sum, SpiceVector3::new(5.0, 7.0, 9.0));

        let dot = v1.dot(&v2);
        assert_eq!(dot, 32.0); // 1*4 + 2*5 + 3*6

        let cross = v1.cross(&v2);
        assert_eq!(cross, SpiceVector3::new(-3.0, 6.0, -3.0));
    }

    #[test]
    fn test_matrix_operations() {
        let identity = SpiceMatrix3x3::identity();
        assert_eq!(identity.determinant(), 1.0);

        let v = SpiceVector3::new(1.0, 2.0, 3.0);
        // Identity matrix times vector should equal the vector
        // We'll implement matrix-vector multiplication in math_core module
    }

    #[test]
    fn test_ephemeris_time() {
        let et1 = EphemerisTime::j2000();
        let et2 = et1 + 3600.0; // Add one hour

        assert_eq!(et1.seconds(), 0.0);
        assert_eq!(et2.seconds(), 3600.0);
        assert_eq!(et2 - et1, 3600.0);
    }
}
EOF

# Create math core module
cat > src/math_core.rs << 'EOF'
//! Core mathematical operations for RustSPICE
//! 
//! This module implements the mathematical functions that are heavily used
//! throughout CSPICE, corresponding to the v*.c and m*.c functions.

use crate::foundation::{
    SpiceDouble, SpiceVector3, SpiceVector6, SpiceMatrix3x3, SpiceMatrix6x6
};
use crate::error_handling::{SpiceResult, SpiceError, SpiceErrorType};

/// Vector addition (equivalent to vadd_c)
pub fn vector_add(v1: &SpiceVector3, v2: &SpiceVector3) -> SpiceVector3 {
    *v1 + *v2
}

/// Vector subtraction (equivalent to vsub_c)
pub fn vector_subtract(v1: &SpiceVector3, v2: &SpiceVector3) -> SpiceVector3 {
    *v1 - *v2
}

/// Vector dot product (equivalent to vdot_c)
pub fn vector_dot(v1: &SpiceVector3, v2: &SpiceVector3) -> SpiceDouble {
    v1.dot(v2)
}

/// Vector cross product (equivalent to vcrss_c)
pub fn vector_cross(v1: &SpiceVector3, v2: &SpiceVector3) -> SpiceVector3 {
    v1.cross(v2)
}

/// Vector norm/magnitude (equivalent to vnorm_c)
pub fn vector_norm(v: &SpiceVector3) -> SpiceDouble {
    v.magnitude()
}

/// Unit vector (equivalent to vhat_c)
pub fn vector_hat(v: &SpiceVector3) -> SpiceResult<SpiceVector3> {
    v.unit()
}

/// Vector scaling (equivalent to vscl_c)
pub fn vector_scale(v: &SpiceVector3, scale: SpiceDouble) -> SpiceVector3 {
    *v * scale
}

/// Vector negation (equivalent to vminus_c)
pub fn vector_minus(v: &SpiceVector3) -> SpiceVector3 {
    *v * (-1.0)
}

/// Linear combination of vectors (equivalent to vlcom_c)
pub fn vector_linear_combination(
    a: SpiceDouble, v1: &SpiceVector3,
    b: SpiceDouble, v2: &SpiceVector3
) -> SpiceVector3 {
    (*v1 * a) + (*v2 * b)
}

/// Three-vector linear combination (equivalent to vlcom3_c)
pub fn vector_linear_combination_3(
    a: SpiceDouble, v1: &SpiceVector3,
    b: SpiceDouble, v2: &SpiceVector3,
    c: SpiceDouble, v3: &SpiceVector3
) -> SpiceVector3 {
    (*v1 * a) + (*v2 * b) + (*v3 * c)
}

/// Matrix-matrix multiplication (equivalent to mxm_c)
pub fn matrix_multiply(m1: &SpiceMatrix3x3, m2: &SpiceMatrix3x3) -> SpiceMatrix3x3 {
    let mut result = [[0.0; 3]; 3];
    
    for i in 0..3 {
        for j in 0..3 {
            for k in 0..3 {
                result[i][j] += m1[i][k] * m2[k][j];
            }
        }
    }
    
    SpiceMatrix3x3::new(result)
}

/// Matrix transpose multiplication (equivalent to mtxm_c)
pub fn matrix_transpose_multiply(m1: &SpiceMatrix3x3, m2: &SpiceMatrix3x3) -> SpiceMatrix3x3 {
    let m1_t = m1.transpose();
    matrix_multiply(&m1_t, m2)
}

/// Matrix-vector multiplication (equivalent to mxv_c)
pub fn matrix_vector_multiply(m: &SpiceMatrix3x3, v: &SpiceVector3) -> SpiceVector3 {
    SpiceVector3::new(
        m[0][0] * v.x() + m[0][1] * v.y() + m[0][2] * v.z(),
        m[1][0] * v.x() + m[1][1] * v.y() + m[1][2] * v.z(),
        m[2][0] * v.x() + m[2][1] * v.y() + m[2][2] * v.z(),
    )
}

/// Matrix transpose vector multiplication (equivalent to mtxv_c)
pub fn matrix_transpose_vector_multiply(m: &SpiceMatrix3x3, v: &SpiceVector3) -> SpiceVector3 {
    let m_t = m.transpose();
    matrix_vector_multiply(&m_t, v)
}

/// Matrix identity (equivalent to ident_c)
pub fn matrix_identity() -> SpiceMatrix3x3 {
    SpiceMatrix3x3::identity()
}

/// Matrix transpose (equivalent to xpose_c)
pub fn matrix_transpose(m: &SpiceMatrix3x3) -> SpiceMatrix3x3 {
    m.transpose()
}

/// Matrix determinant (equivalent to det_c)
pub fn matrix_determinant(m: &SpiceMatrix3x3) -> SpiceDouble {
    m.determinant()
}

/// Matrix inverse (equivalent to invert_c)
pub fn matrix_invert(m: &SpiceMatrix3x3) -> SpiceResult<SpiceMatrix3x3> {
    let det = m.determinant();
    
    if det.abs() < 1e-15 {
        return Err(SpiceError::new(
            SpiceErrorType::ComputationError,
            "Matrix is singular (determinant is zero)".to_string()
        ));
    }
    
    let inv_det = 1.0 / det;
    let mut inv = [[0.0; 3]; 3];
    
    // Calculate adjugate matrix and divide by determinant
    inv[0][0] = (m[1][1] * m[2][2] - m[1][2] * m[2][1]) * inv_det;
    inv[0][1] = (m[0][2] * m[2][1] - m[0][1] * m[2][2]) * inv_det;
    inv[0][2] = (m[0][1] * m[1][2] - m[0][2] * m[1][1]) * inv_det;
    
    inv[1][0] = (m[1][2] * m[2][0] - m[1][0] * m[2][2]) * inv_det;
    inv[1][1] = (m[0][0] * m[2][2] - m[0][2] * m[2][0]) * inv_det;
    inv[1][2] = (m[0][2] * m[1][0] - m[0][0] * m[1][2]) * inv_det;
    
    inv[2][0] = (m[1][0] * m[2][1] - m[1][1] * m[2][0]) * inv_det;
    inv[2][1] = (m[0][1] * m[2][0] - m[0][0] * m[2][1]) * inv_det;
    inv[2][2] = (m[0][0] * m[1][1] - m[0][1] * m[1][0]) * inv_det;
    
    Ok(SpiceMatrix3x3::new(inv))
}

/// Angle between vectors (equivalent to vsep_c)
pub fn vector_separation(v1: &SpiceVector3, v2: &SpiceVector3) -> SpiceResult<SpiceDouble> {
    let mag1 = v1.magnitude();
    let mag2 = v2.magnitude();
    
    if mag1 == 0.0 || mag2 == 0.0 {
        return Err(SpiceError::new(
            SpiceErrorType::ComputationError,
            "Cannot compute angle between zero vectors".to_string()
        ));
    }
    
    let cos_angle = v1.dot(v2) / (mag1 * mag2);
    
    // Clamp to avoid numerical errors with acos
    let cos_angle = cos_angle.max(-1.0).min(1.0);
    
    Ok(cos_angle.acos())
}

/// Distance between vectors (equivalent to vdist_c)
pub fn vector_distance(v1: &SpiceVector3, v2: &SpiceVector3) -> SpiceDouble {
    (*v1 - *v2).magnitude()
}

/// Check if vectors are perpendicular within tolerance
pub fn vectors_perpendicular(v1: &SpiceVector3, v2: &SpiceVector3, tolerance: SpiceDouble) -> bool {
    let dot_product = v1.dot(v2).abs();
    let magnitude_product = v1.magnitude() * v2.magnitude();
    
    if magnitude_product == 0.0 {
        return false;
    }
    
    dot_product / magnitude_product < tolerance
}

/// Check if vectors are parallel within tolerance
pub fn vectors_parallel(v1: &SpiceVector3, v2: &SpiceVector3, tolerance: SpiceDouble) -> bool {
    match vector_separation(v1, v2) {
        Ok(angle) => angle < tolerance || (core::f64::consts::PI - angle) < tolerance,
        Err(_) => false,
    }
}

/// Mathematical constants
pub mod constants {
    use crate::foundation::SpiceDouble;
    
    /// PI
    pub const PI: SpiceDouble = core::f64::consts::PI;
    
    /// Two times PI
    pub const TWO_PI: SpiceDouble = 2.0 * PI;
    
    /// Half PI
    pub const HALF_PI: SpiceDouble = PI / 2.0;
    
    /// Degrees per radian
    pub const DEGREES_PER_RADIAN: SpiceDouble = 180.0 / PI;
    
    /// Radians per degree
    pub const RADIANS_PER_DEGREE: SpiceDouble = PI / 180.0;
    
    /// Speed of light in km/s
    pub const SPEED_OF_LIGHT: SpiceDouble = 299792.458;
    
    /// Astronomical unit in km
    pub const ASTRONOMICAL_UNIT: SpiceDouble = 149597870.7;
    
    /// Julian year in seconds
    pub const JULIAN_YEAR: SpiceDouble = 365.25 * 86400.0;
}

/// Convert degrees to radians
pub fn degrees_to_radians(degrees: SpiceDouble) -> SpiceDouble {
    degrees * constants::RADIANS_PER_DEGREE
}

/// Convert radians to degrees
pub fn radians_to_degrees(radians: SpiceDouble) -> SpiceDouble {
    radians * constants::DEGREES_PER_RADIAN
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::foundation::SpiceVector3;

    #[test]
    fn test_vector_operations() {
        let v1 = SpiceVector3::new(1.0, 0.0, 0.0);
        let v2 = SpiceVector3::new(0.0, 1.0, 0.0);
        
        assert_eq!(vector_dot(&v1, &v2), 0.0);
        assert_eq!(vector_cross(&v1, &v2), SpiceVector3::new(0.0, 0.0, 1.0));
        assert!((vector_separation(&v1, &v2).unwrap() - constants::HALF_PI).abs() < 1e-10);
    }

    #[test]
    fn test_matrix_operations() {
        let identity = matrix_identity();
        let v = SpiceVector3::new(1.0, 2.0, 3.0);
        
        let result = matrix_vector_multiply(&identity, &v);
        assert_eq!(result, v);
        
        assert!((matrix_determinant(&identity) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_matrix_inverse() {
        let m = SpiceMatrix3x3::new([
            [2.0, 0.0, 0.0],
            [0.0, 2.0, 0.0],
            [0.0, 0.0, 2.0],
        ]);
        
        let inv = matrix_invert(&m).unwrap();
        let expected = SpiceMatrix3x3::new([
            [0.5, 0.0, 0.0],
            [0.0, 0.5, 0.0],
            [0.0, 0.0, 0.5],
        ]);
        
        // Check that result is close to expected
        for i in 0..3 {
            for j in 0..3 {
                assert!((inv[i][j] - expected[i][j]).abs() < 1e-10);
            }
        }
    }
}
EOF

echo "✅ Phase 1 foundation modules created!"
echo ""
echo "📊 Created modules:"
echo "   - src/error_handling.rs (SpiceError, SpiceResult, error tracing)"
echo "   - src/foundation.rs (Core data types, vectors, matrices)"
echo "   - src/math_core.rs (Mathematical operations)"
echo ""
echo "🚀 Next steps:"
echo "   1. Implement time_system module (str2et_c, et2utc_c, etc.)"
echo "   2. Create coordinate transformation module"
echo "   3. Build virtual file system for kernel loading"
echo "   4. Start converting core CSPICE functions"
echo ""
echo "📝 To continue with Phase 2 (Time System):"
echo "   ./setup-phase2-time-system.sh"
