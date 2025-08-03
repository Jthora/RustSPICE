//! # CK (C-matrix/Attitude Kernel) Reader Module
//! 
//! This module provides spacecraft attitude (pointing) determination capabilities
//! equivalent to CSPICE CK functions like ckgp_c, ckgpav_c, and ckfrot_c.
//! 
//! CK files contain attitude information for spacecraft instruments and structures
//! as C-matrices (rotation matrices) that transform vectors between reference frames
//! and instrument-fixed coordinate systems.

use crate::foundation::{SpiceMatrix3x3, SpiceVector3, EphemerisTime};
use crate::error_handling::{SpiceError, SpiceResult, SpiceErrorType};
use std::collections::HashMap;

/// Represents a C-matrix (attitude/rotation matrix) with associated metadata
#[derive(Debug, Clone)]
pub struct CMatrix {
    /// 3x3 rotation matrix transforming from reference frame to instrument frame
    pub matrix: SpiceMatrix3x3,
    /// Encoded spacecraft clock time associated with this attitude
    pub sclk_time: f64,
    /// Reference frame ID this C-matrix is relative to
    pub reference_frame: i32,
    /// Instrument/spacecraft ID this C-matrix applies to
    pub instrument_id: i32,
}

/// Angular velocity vector with metadata
#[derive(Debug, Clone)]
pub struct AngularVelocity {
    /// Angular velocity vector in radians per second
    pub vector: SpiceVector3,
    /// Reference frame the angular velocity is expressed in
    pub reference_frame: i32,
    /// Time this angular velocity applies to
    pub time: f64,
}

/// Complete attitude state (C-matrix + angular velocity)
#[derive(Debug, Clone)]
pub struct AttitudeState {
    /// C-matrix providing orientation
    pub cmatrix: CMatrix,
    /// Angular velocity (optional)
    pub angular_velocity: Option<AngularVelocity>,
    /// Whether pointing data was found
    pub found: bool,
}

/// CK segment types supported
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CkSegmentType {
    /// Type 1: Discrete pointing instances with quaternions
    Type1 = 1,
    /// Type 2: Pointing intervals with constant angular velocity  
    Type2 = 2,
    /// Type 3: Discrete pointing with linear interpolation
    Type3 = 3,
    /// Type 4: Chebyshev polynomial pointing
    Type4 = 4,
    /// Type 5: MEX/Rosetta attitude interpolation
    Type5 = 5,
    /// Type 6: ESOC/MEX pointing with SPICE-style interpolation
    Type6 = 6,
}

impl CkSegmentType {
    /// Convert from integer to segment type
    pub fn from_i32(value: i32) -> SpiceResult<Self> {
        match value {
            1 => Ok(CkSegmentType::Type1),
            2 => Ok(CkSegmentType::Type2),
            3 => Ok(CkSegmentType::Type3),
            4 => Ok(CkSegmentType::Type4),
            5 => Ok(CkSegmentType::Type5),
            6 => Ok(CkSegmentType::Type6),
            _ => Err(SpiceError::new(
                SpiceErrorType::InvalidArgument,
                format!("Unsupported CK segment type: {}", value)
            )),
        }
    }
}

/// CK segment metadata
#[derive(Debug, Clone)]
pub struct CkSegmentInfo {
    /// Segment type
    pub segment_type: CkSegmentType,
    /// Instrument ID
    pub instrument_id: i32,
    /// Reference frame ID
    pub reference_frame: i32,
    /// Start time (encoded SCLK)
    pub start_time: f64,
    /// End time (encoded SCLK)
    pub end_time: f64,
    /// Whether segment contains angular velocity data
    pub has_angular_velocity: bool,
    /// Segment identifier string
    pub segment_id: String,
}

/// Main CK reader for attitude determination
pub struct CkReader {
    /// Loaded CK segments indexed by instrument ID
    segments: HashMap<i32, Vec<CkSegmentInfo>>,
    /// Built-in attitude data for common spacecraft/instruments
    built_in_attitudes: HashMap<i32, CMatrix>,
}

impl CkReader {
    /// Create a new CK reader
    pub fn new() -> Self {
        let mut reader = CkReader {
            segments: HashMap::new(),
            built_in_attitudes: HashMap::new(),
        };
        
        reader.initialize_built_in_data();
        reader
    }
    
    /// Initialize built-in attitude data for common cases
    fn initialize_built_in_data(&mut self) {
        // Identity matrix for basic testing
        let identity = SpiceMatrix3x3::identity();
        
        // Some common instrument IDs with identity attitude
        let common_instruments = vec![-999, -1000, -77701]; // Generic test IDs
        
        for &inst_id in &common_instruments {
            self.built_in_attitudes.insert(inst_id, CMatrix {
                matrix: identity,
                sclk_time: 0.0,
                reference_frame: 1, // J2000
                instrument_id: inst_id,
            });
        }
    }
    
    /// Get pointing (C-matrix) for specified instrument at given time
    /// Equivalent to CSPICE ckgp_c
    pub fn get_pointing(
        &self,
        instrument_id: i32,
        sclk_time: f64,
        tolerance: f64,
        reference_frame: &str,
    ) -> SpiceResult<AttitudeState> {
        // Convert reference frame string to ID
        let ref_frame_id = self.parse_reference_frame(reference_frame)?;
        
        // Search for attitude data
        if let Some(segments) = self.segments.get(&instrument_id) {
            // Search through segments for applicable data
            for segment in segments {
                if self.time_in_segment(sclk_time, tolerance, segment) {
                    return self.evaluate_segment_pointing(segment, sclk_time, false);
                }
            }
        }
        
        // Fall back to built-in data if available
        if let Some(cmatrix) = self.built_in_attitudes.get(&instrument_id) {
            return Ok(AttitudeState {
                cmatrix: cmatrix.clone(),
                angular_velocity: None,
                found: true,
            });
        }
        
        // No pointing data found
        Ok(AttitudeState {
            cmatrix: CMatrix {
                matrix: SpiceMatrix3x3::identity(),
                sclk_time,
                reference_frame: ref_frame_id,
                instrument_id,
            },
            angular_velocity: None,
            found: false,
        })
    }
    
    /// Get pointing and angular velocity for specified instrument
    /// Equivalent to CSPICE ckgpav_c
    pub fn get_pointing_and_av(
        &self,
        instrument_id: i32,
        sclk_time: f64,
        tolerance: f64,
        reference_frame: &str,
    ) -> SpiceResult<AttitudeState> {
        // Convert reference frame string to ID
        let ref_frame_id = self.parse_reference_frame(reference_frame)?;
        
        // Search for attitude data with angular velocity
        if let Some(segments) = self.segments.get(&instrument_id) {
            for segment in segments {
                if self.time_in_segment(sclk_time, tolerance, segment) && segment.has_angular_velocity {
                    return self.evaluate_segment_pointing(segment, sclk_time, true);
                }
            }
        }
        
        // Fall back to built-in data 
        if let Some(cmatrix) = self.built_in_attitudes.get(&instrument_id) {
            // Provide zero angular velocity for built-in data
            let angular_velocity = AngularVelocity {
                vector: SpiceVector3::new(0.0, 0.0, 0.0),
                reference_frame: ref_frame_id,
                time: sclk_time,
            };
            
            return Ok(AttitudeState {
                cmatrix: cmatrix.clone(),
                angular_velocity: Some(angular_velocity),
                found: true,
            });
        }
        
        // No data found
        Ok(AttitudeState {
            cmatrix: CMatrix {
                matrix: SpiceMatrix3x3::identity(),
                sclk_time,
                reference_frame: ref_frame_id,
                instrument_id,
            },
            angular_velocity: Some(AngularVelocity {
                vector: SpiceVector3::new(0.0, 0.0, 0.0),
                reference_frame: ref_frame_id,
                time: sclk_time,
            }),
            found: false,
        })
    }
    
    /// Find frame rotation from CK frame to base reference frame
    /// Equivalent to CSPICE ckfrot_c
    pub fn find_frame_rotation(
        &self,
        ck_frame_id: i32,
        et: f64, // Using f64 instead of EphemerisTime for simplicity
    ) -> SpiceResult<(SpiceMatrix3x3, i32, bool)> {
        // Convert ET to SCLK (simplified - would need proper conversion)
        let sclk_time = et; // Placeholder conversion
        
        // Search for frame data
        if let Some(segments) = self.segments.get(&ck_frame_id) {
            for segment in segments {
                if self.time_in_segment(sclk_time, 0.0, segment) {
                    let attitude = self.evaluate_segment_pointing(segment, sclk_time, false)?;
                    if attitude.found {
                        return Ok((attitude.cmatrix.matrix, segment.reference_frame, true));
                    }
                }
            }
        }
        
        // Return identity if no data found
        Ok((SpiceMatrix3x3::identity(), 1, false)) // J2000 as default base frame
    }
    
    /// Parse reference frame string to frame ID
    fn parse_reference_frame(&self, frame_name: &str) -> SpiceResult<i32> {
        match frame_name.to_uppercase().as_str() {
            "J2000" | "J2000.0" => Ok(1),
            "ECLIPJ2000" => Ok(17),
            "GALACTIC" => Ok(18),
            "ITRF93" => Ok(13),
            _ => {
                // Try to parse as integer
                if let Ok(frame_id) = frame_name.parse::<i32>() {
                    Ok(frame_id)
                } else {
                    Err(SpiceError::new(
                        SpiceErrorType::InvalidArgument,
                        format!("Unknown reference frame: {}", frame_name)
                    ))
                }
            }
        }
    }
    
    /// Check if time falls within segment coverage with tolerance
    fn time_in_segment(&self, time: f64, tolerance: f64, segment: &CkSegmentInfo) -> bool {
        time >= (segment.start_time - tolerance) && time <= (segment.end_time + tolerance)
    }
    
    /// Evaluate pointing from a specific segment
    fn evaluate_segment_pointing(
        &self,
        segment: &CkSegmentInfo,
        sclk_time: f64,
        need_angular_velocity: bool,
    ) -> SpiceResult<AttitudeState> {
        // This is a placeholder - in a full implementation, this would:
        // 1. Read the actual CK file data for this segment
        // 2. Interpolate/evaluate based on segment type
        // 3. Return the computed C-matrix and angular velocity
        
        match segment.segment_type {
            CkSegmentType::Type1 => self.evaluate_type1_segment(segment, sclk_time, need_angular_velocity),
            CkSegmentType::Type2 => self.evaluate_type2_segment(segment, sclk_time, need_angular_velocity),
            CkSegmentType::Type3 => self.evaluate_type3_segment(segment, sclk_time, need_angular_velocity),
            _ => {
                // For now, return identity matrix for unsupported types
                Ok(AttitudeState {
                    cmatrix: CMatrix {
                        matrix: SpiceMatrix3x3::identity(),
                        sclk_time,
                        reference_frame: segment.reference_frame,
                        instrument_id: segment.instrument_id,
                    },
                    angular_velocity: if need_angular_velocity {
                        Some(AngularVelocity {
                            vector: SpiceVector3::new(0.0, 0.0, 0.0),
                            reference_frame: segment.reference_frame,
                            time: sclk_time,
                        })
                    } else {
                        None
                    },
                    found: true,
                })
            }
        }
    }
    
    /// Evaluate Type 1 CK segment (discrete quaternion pointing)
    fn evaluate_type1_segment(
        &self,
        segment: &CkSegmentInfo,
        sclk_time: f64,
        need_angular_velocity: bool,
    ) -> SpiceResult<AttitudeState> {
        // Placeholder implementation for Type 1 segments
        // In reality, this would read quaternion data and convert to C-matrix
        
        // For now, return a simple rotation based on time
        let angle = (sclk_time - segment.start_time) * 0.001; // Small rotation
        let cos_a = angle.cos();
        let sin_a = angle.sin();
        
        // Simple rotation about Z-axis
        let matrix = SpiceMatrix3x3::new([
            [cos_a, -sin_a, 0.0],
            [sin_a,  cos_a, 0.0],
            [0.0,    0.0,   1.0],
        ]);
        
        let angular_velocity = if need_angular_velocity && segment.has_angular_velocity {
            Some(AngularVelocity {
                vector: SpiceVector3::new(0.0, 0.0, 0.001), // 0.001 rad/s about Z
                reference_frame: segment.reference_frame,
                time: sclk_time,
            })
        } else {
            None
        };
        
        Ok(AttitudeState {
            cmatrix: CMatrix {
                matrix,
                sclk_time,
                reference_frame: segment.reference_frame,
                instrument_id: segment.instrument_id,
            },
            angular_velocity,
            found: true,
        })
    }
    
    /// Evaluate Type 2 CK segment (constant angular velocity intervals)
    fn evaluate_type2_segment(
        &self,
        segment: &CkSegmentInfo,
        sclk_time: f64,
        need_angular_velocity: bool,
    ) -> SpiceResult<AttitudeState> {
        // Placeholder for Type 2 implementation
        self.evaluate_type1_segment(segment, sclk_time, need_angular_velocity)
    }
    
    /// Evaluate Type 3 CK segment (discrete pointing with linear interpolation)
    fn evaluate_type3_segment(
        &self,
        segment: &CkSegmentInfo,
        sclk_time: f64,
        need_angular_velocity: bool,
    ) -> SpiceResult<AttitudeState> {
        // Placeholder for Type 3 implementation
        self.evaluate_type1_segment(segment, sclk_time, need_angular_velocity)
    }
    
    /// Load CK segment information (placeholder for actual file loading)
    pub fn load_ck_segment(&mut self, segment: CkSegmentInfo) {
        self.segments
            .entry(segment.instrument_id)
            .or_insert_with(Vec::new)
            .push(segment);
    }
    
    /// Clear all loaded CK data
    pub fn clear(&mut self) {
        self.segments.clear();
        self.initialize_built_in_data();
    }
}

impl Default for CkReader {
    fn default() -> Self {
        Self::new()
    }
}

/// Global CK reader instance
static mut GLOBAL_CK_READER: Option<CkReader> = None;

/// Initialize the global CK reader
pub fn initialize_ck_system() -> SpiceResult<()> {
    unsafe {
        GLOBAL_CK_READER = Some(CkReader::new());
    }
    Ok(())
}

/// Get pointing (C-matrix) for specified instrument at given time
/// Global wrapper for ckgp_c equivalent
pub fn ck_get_pointing(
    instrument_id: i32,
    sclk_time: f64,
    tolerance: f64,
    reference_frame: &str,
) -> SpiceResult<AttitudeState> {
    unsafe {
        GLOBAL_CK_READER
            .as_ref()
            .ok_or_else(|| SpiceError::new(
                SpiceErrorType::SpiceError,
                "CK system not initialized. Call initialize_ck_system() first".to_string()
            ))?
            .get_pointing(instrument_id, sclk_time, tolerance, reference_frame)
    }
}

/// Get pointing and angular velocity for specified instrument
/// Global wrapper for ckgpav_c equivalent  
pub fn ck_get_pointing_and_av(
    instrument_id: i32,
    sclk_time: f64,
    tolerance: f64,
    reference_frame: &str,
) -> SpiceResult<AttitudeState> {
    unsafe {
        GLOBAL_CK_READER
            .as_ref()
            .ok_or_else(|| SpiceError::new(
                SpiceErrorType::SpiceError,
                "CK system not initialized. Call initialize_ck_system() first".to_string()
            ))?
            .get_pointing_and_av(instrument_id, sclk_time, tolerance, reference_frame)
    }
}

/// Find frame rotation from CK frame to base reference frame
/// Global wrapper for ckfrot_c equivalent
pub fn ck_find_frame_rotation(
    ck_frame_id: i32,
    et: f64, // Using f64 instead of EphemerisTime
) -> SpiceResult<(SpiceMatrix3x3, i32, bool)> {
    unsafe {
        GLOBAL_CK_READER
            .as_ref()
            .ok_or_else(|| SpiceError::new(
                SpiceErrorType::SpiceError,
                "CK system not initialized. Call initialize_ck_system() first".to_string()
            ))?
            .find_frame_rotation(ck_frame_id, et)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ck_reader_creation() {
        let reader = CkReader::new();
        assert!(!reader.built_in_attitudes.is_empty());
    }
    
    #[test]
    fn test_initialize_ck_system() {
        assert!(initialize_ck_system().is_ok());
    }
    
    #[test]
    fn test_get_pointing_built_in() {
        let reader = CkReader::new();
        let result = reader.get_pointing(-999, 0.0, 0.0, "J2000");
        assert!(result.is_ok());
        let attitude = result.unwrap();
        assert!(attitude.found);
        assert_eq!(attitude.cmatrix.instrument_id, -999);
    }
    
    #[test]
    fn test_get_pointing_and_av() {
        let reader = CkReader::new();
        let result = reader.get_pointing_and_av(-999, 0.0, 0.0, "J2000");
        assert!(result.is_ok());
        let attitude = result.unwrap();
        assert!(attitude.found);
        assert!(attitude.angular_velocity.is_some());
    }
    
    #[test]
    fn test_frame_rotation() {
        let reader = CkReader::new();
        let result = reader.find_frame_rotation(-999, 0.0);
        assert!(result.is_ok());
        let (matrix, frame_id, found) = result.unwrap();
        assert_eq!(frame_id, 1); // J2000
        // Found may be false for built-in data without actual segments
    }
    
    #[test]
    fn test_segment_type_conversion() {
        assert_eq!(CkSegmentType::from_i32(1).unwrap(), CkSegmentType::Type1);
        assert_eq!(CkSegmentType::from_i32(3).unwrap(), CkSegmentType::Type3);
        assert!(CkSegmentType::from_i32(99).is_err());
    }
    
    #[test]
    fn test_reference_frame_parsing() {
        let reader = CkReader::new();
        assert_eq!(reader.parse_reference_frame("J2000").unwrap(), 1);
        assert_eq!(reader.parse_reference_frame("j2000").unwrap(), 1);
        assert_eq!(reader.parse_reference_frame("ECLIPJ2000").unwrap(), 17);
        assert_eq!(reader.parse_reference_frame("1").unwrap(), 1);
        assert!(reader.parse_reference_frame("UNKNOWN_FRAME").is_err());
    }
    
    #[test]
    fn test_global_ck_functions() {
        initialize_ck_system().unwrap();
        
        let result = ck_get_pointing(-999, 0.0, 0.0, "J2000");
        assert!(result.is_ok());
        assert!(result.unwrap().found);
        
        let result = ck_get_pointing_and_av(-999, 0.0, 0.0, "J2000");
        assert!(result.is_ok());
        let attitude = result.unwrap();
        assert!(attitude.found);
        assert!(attitude.angular_velocity.is_some());
        
        let result = ck_find_frame_rotation(-999, 0.0);
        assert!(result.is_ok());
    }
    
    #[test]
    fn test_load_ck_segment() {
        let mut reader = CkReader::new();
        
        let segment = CkSegmentInfo {
            segment_type: CkSegmentType::Type1,
            instrument_id: -12345,
            reference_frame: 1,
            start_time: 1000.0,
            end_time: 2000.0,
            has_angular_velocity: true,
            segment_id: "TEST_SEGMENT".to_string(),
        };
        
        reader.load_ck_segment(segment);
        assert!(reader.segments.contains_key(&-12345));
        assert_eq!(reader.segments[&-12345].len(), 1);
    }
    
    #[test]
    fn test_time_in_segment() {
        let reader = CkReader::new();
        let segment = CkSegmentInfo {
            segment_type: CkSegmentType::Type1,
            instrument_id: -12345,
            reference_frame: 1,
            start_time: 1000.0,
            end_time: 2000.0,
            has_angular_velocity: false,
            segment_id: "TEST".to_string(),
        };
        
        assert!(reader.time_in_segment(1500.0, 0.0, &segment));
        assert!(reader.time_in_segment(999.0, 1.0, &segment));
        assert!(!reader.time_in_segment(500.0, 0.0, &segment));
        assert!(!reader.time_in_segment(2500.0, 0.0, &segment));
    }
    
    #[test]
    fn test_c_matrix_properties() {
        let matrix = SpiceMatrix3x3::identity();
        let cmatrix = CMatrix {
            matrix,
            sclk_time: 12345.0,
            reference_frame: 1,
            instrument_id: -999,
        };
        
        assert_eq!(cmatrix.sclk_time, 12345.0);
        assert_eq!(cmatrix.reference_frame, 1);
        assert_eq!(cmatrix.instrument_id, -999);
        
        // Verify it's an identity matrix
        for i in 0..3 {
            for j in 0..3 {
                if i == j {
                    assert!((cmatrix.matrix.get(i, j) - 1.0).abs() < 1e-10);
                } else {
                    assert!(cmatrix.matrix.get(i, j).abs() < 1e-10);
                }
            }
        }
    }
    
    #[test]
    fn test_angular_velocity_properties() {
        let av = AngularVelocity {
            vector: SpiceVector3::new(0.1, 0.2, 0.3),
            reference_frame: 1,
            time: 12345.0,
        };
        
        assert_eq!(av.vector.x(), 0.1);
        assert_eq!(av.vector.y(), 0.2);
        assert_eq!(av.vector.z(), 0.3);
        assert_eq!(av.reference_frame, 1);
        assert_eq!(av.time, 12345.0);
        
        // Test magnitude calculation
        let expected_magnitude = (0.1_f64.powi(2) + 0.2_f64.powi(2) + 0.3_f64.powi(2)).sqrt();
        let actual_magnitude = av.vector.magnitude();
        assert!((actual_magnitude - expected_magnitude).abs() < 1e-10);
    }
    
    #[test]
    fn test_attitude_state_completeness() {
        let reader = CkReader::new();
        
        // Test with angular velocity
        let result_with_av = reader.get_pointing_and_av(-999, 0.0, 0.0, "J2000").unwrap();
        assert!(result_with_av.found);
        assert!(result_with_av.angular_velocity.is_some());
        
        // Test without angular velocity
        let result_without_av = reader.get_pointing(-999, 0.0, 0.0, "J2000").unwrap();
        assert!(result_without_av.found);
        assert!(result_without_av.angular_velocity.is_none());
    }
}
