//! RustSPICE: Complete Rust implementation of NASA's SPICE toolkit
//! 
//! This is a from-scratch conversion of the entire CSPICE library to Rust,
//! designed for WebAssembly compatibility while maintaining full numerical
//! accuracy and functional equivalence with the original CSPICE.

#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::string::String;
#[cfg(not(feature = "std"))]
use alloc::format;

#[cfg(feature = "std")]
use std::string::String;
#[cfg(feature = "std")]
use std::format;

// Core modules
pub mod error_handling;
pub mod math_core;
pub mod foundation;
pub mod time_system;
pub mod coordinates;
pub mod file_system;
pub mod kernel_system;
pub mod kernel_pool;
pub mod advanced_math;
pub mod spk_reader;
pub mod ephemeris;
pub mod body_data;
pub mod ck_reader;

// Comprehensive integration and stress tests
#[cfg(test)]
pub mod comprehensive_tests;

// Re-export the most important types and functions
pub use error_handling::{SpiceError, SpiceResult, ErrorTrace};
pub use foundation::{
    SpiceDouble, SpiceInt, SpiceChar, SpiceBoolean,
    SpiceMatrix3x3, SpiceMatrix6x6, SpiceVector3, SpiceVector6,
    StateVector, EphemerisTime, JulianDate
};
pub use time_system::{
    str_to_et, et_to_utc, time_parse, time_output, delta_et_utc,
    ParsedTime, CalendarType, Era, is_leap_year,
    day_of_year_to_month_day, month_day_to_day_of_year
};
pub use ephemeris::{ephemeris_state, ephemeris_position};
pub use kernel_system::{furnish_kernel, unload_kernel, clear_kernels};
pub use ck_reader::{
    ck_get_pointing, ck_get_pointing_and_av, ck_find_frame_rotation,
    initialize_ck_system, CMatrix, AngularVelocity, AttitudeState,
    CkSegmentType, CkSegmentInfo
};

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
    spk_reader::initialize_spk_reader()?;
    ck_reader::initialize_ck_system()?;
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

// Coordinate System Exports
pub use coordinates::{
    get_position_transformation, get_state_transformation,
    rotate_vector, rotation_matrix_axis_angle, axis_angle_rotation,
    matrix_to_euler, euler_to_matrix, transform_position, transform_state,
    rotation_between_vectors, is_rotation_matrix,
    ReferenceFrame, EulerSequence, RotationAxis, SpacecraftOrientation
};
