//! Celestial body data and ID management for RustSPICE

use crate::foundation::SpiceInt;
use crate::error_handling::{SpiceResult, SpiceError, SpiceErrorType};

/// Convert body name to NAIF ID (equivalent to bodn2c_c)
pub fn body_name_to_code(_name: &str) -> SpiceResult<SpiceInt> {
    // TODO: Implement body name to ID conversion
    Err(SpiceError::new(
        SpiceErrorType::InvalidTarget,
        "Body data system not yet implemented".into()
    ))
}

/// Convert NAIF ID to body name (equivalent to bodc2n_c)
pub fn body_code_to_name(_code: SpiceInt) -> SpiceResult<String> {
    // TODO: Implement ID to body name conversion
    Err(SpiceError::new(
        SpiceErrorType::InvalidTarget,
        "Body data system not yet implemented".into()
    ))
}
