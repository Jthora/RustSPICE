//! Core mathematical operations for RustSPICE
//! 
//! This module implements the mathematical functions that are heavily used
//! throughout CSPICE, corresponding to the v*.c and m*.c functions.

use crate::foundation::{
    SpiceDouble, SpiceVector3, SpiceMatrix3x3
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
            "Matrix is singular (determinant is zero)".into()
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
            "Cannot compute angle between zero vectors".into()
        ));
    }
    
    let cos_angle = v1.dot(v2) / (mag1 * mag2);
    
    // Clamp to avoid numerical errors with acos
    let cos_angle = cos_angle.max(-1.0).min(1.0);
    
    Ok(libm::acos(cos_angle))
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
