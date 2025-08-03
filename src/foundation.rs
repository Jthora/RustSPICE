//! Foundation data types for RustSPICE
//! 
//! This module defines the core data structures that correspond to SPICE
//! data types and provides the foundation for all other modules.

#[cfg(not(feature = "std"))]
use alloc::string::String;

#[cfg(feature = "std")]
use std::string::String;

use core::ops::{Add, Sub, Mul, Index, IndexMut};
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

    /// Get matrix element
    pub fn get(&self, row: usize, col: usize) -> SpiceDouble {
        self.0[row][col]
    }

    /// Set matrix element
    pub fn set(&mut self, row: usize, col: usize, value: SpiceDouble) {
        self.0[row][col] = value;
    }

    /// Matrix multiplication
    pub fn multiply(&self, other: &SpiceMatrix3x3) -> SpiceMatrix3x3 {
        let mut result = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                for k in 0..3 {
                    result[i][j] += self.0[i][k] * other.0[k][j];
                }
            }
        }
        SpiceMatrix3x3(result)
    }

    /// Matrix subtraction
    pub fn subtract(&self, other: &SpiceMatrix3x3) -> SpiceMatrix3x3 {
        let mut result = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                result[i][j] = self.0[i][j] - other.0[i][j];
            }
        }
        SpiceMatrix3x3(result)
    }

    /// Scale matrix by scalar
    pub fn scale(&self, scalar: SpiceDouble) -> SpiceMatrix3x3 {
        let mut result = [[0.0; 3]; 3];
        for i in 0..3 {
            for j in 0..3 {
                result[i][j] = self.0[i][j] * scalar;
            }
        }
        SpiceMatrix3x3(result)
    }

    /// Multiply matrix by vector
    pub fn multiply_vector(&self, vector: &SpiceVector3) -> SpiceVector3 {
        SpiceVector3([
            self.0[0][0] * vector.0[0] + self.0[0][1] * vector.0[1] + self.0[0][2] * vector.0[2],
            self.0[1][0] * vector.0[0] + self.0[1][1] * vector.0[1] + self.0[1][2] * vector.0[2],
            self.0[2][0] * vector.0[0] + self.0[2][1] * vector.0[1] + self.0[2][2] * vector.0[2],
        ])
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

    /// Get matrix element
    pub fn get(&self, row: usize, col: usize) -> SpiceDouble {
        self.0[row][col]
    }

    /// Set matrix element
    pub fn set(&mut self, row: usize, col: usize, value: SpiceDouble) {
        self.0[row][col] = value;
    }

    /// Multiply matrix by 6D vector
    pub fn multiply_vector(&self, vector: &SpiceVector6) -> SpiceVector6 {
        let mut result = [0.0; 6];
        for i in 0..6 {
            for j in 0..6 {
                result[i] += self.0[i][j] * vector.0[j];
            }
        }
        SpiceVector6(result)
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
        libm::sqrt(self.0[0] * self.0[0] + self.0[1] * self.0[1] + self.0[2] * self.0[2])
    }

    /// Unit vector (normalized)
    pub fn unit(&self) -> SpiceResult<SpiceVector3> {
        let mag = self.magnitude();
        if mag == 0.0 {
            return Err(SpiceError::new(
                SpiceErrorType::ComputationError,
                "Cannot normalize zero vector".into()
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

    /// Normalize vector (alias for unit)
    pub fn normalize(&self) -> SpiceResult<SpiceVector3> {
        self.unit()
    }

    /// Scale vector by scalar
    pub fn scale(&self, scalar: SpiceDouble) -> SpiceVector3 {
        SpiceVector3([
            self.0[0] * scalar,
            self.0[1] * scalar,
            self.0[2] * scalar,
        ])
    }

    /// Subtract two vectors
    pub fn subtract(&self, other: &SpiceVector3) -> SpiceVector3 {
        SpiceVector3([
            self.0[0] - other.0[0],
            self.0[1] - other.0[1],
            self.0[2] - other.0[2],
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

    /// Get vector element
    pub fn get(&self, index: usize) -> SpiceDouble {
        self.0[index]
    }

    /// Set vector element
    pub fn set(&mut self, index: usize, value: SpiceDouble) {
        self.0[index] = value;
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
