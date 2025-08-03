//! Error handling system for RustSPICE
//! 
//! This module replaces CSPICE's error handling system (chkin_/chkout_/sigerr_)
//! with Rust's native Result type and structured error handling.

#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::vec::Vec;
#[cfg(not(feature = "std"))]
use alloc::format;

#[cfg(feature = "std")]
use std::string::String;
#[cfg(feature = "std")]
use std::vec::Vec;
#[cfg(feature = "std")]
use std::format;

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
    /// Kernel already loaded error
    KernelAlreadyLoaded,
    /// Kernel loading error
    KernelLoadError,
    /// Too many kernels loaded
    TooManyKernels,
    /// Invalid kernel path
    InvalidKernelPath,
    /// Invalid kernel data format
    InvalidKernelData,
    /// File read error
    FileReadError,
    /// Kernel pool not initialized error
    PoolNotInitialized,
    /// Invalid data type for kernel pool operations  
    InvalidDataType,
    /// Invalid format (e.g., malformed text kernel)
    InvalidFormat,
    /// Invalid index (e.g., array index out of bounds)
    InvalidIndex,
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
            SpiceErrorType::KernelAlreadyLoaded => "KERNEL_ALREADY_LOADED",
            SpiceErrorType::TooManyKernels => "TOO_MANY_KERNELS",
            SpiceErrorType::InvalidKernelPath => "INVALID_KERNEL_PATH",
            SpiceErrorType::InvalidKernelData => "INVALID_KERNEL_DATA",
            SpiceErrorType::FileReadError => "FILE_READ_ERROR",
            SpiceErrorType::PoolNotInitialized => "POOL_NOT_INITIALIZED",
            SpiceErrorType::InvalidDataType => "INVALID_DATA_TYPE",
            SpiceErrorType::InvalidFormat => "INVALID_FORMAT",
            SpiceErrorType::InvalidIndex => "INVALID_INDEX",
            SpiceErrorType::SpiceError => "SPICE_ERROR",
            SpiceErrorType::KernelLoadError => "KERNEL_LOAD_ERROR",
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
