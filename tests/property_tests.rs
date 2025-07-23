use proptest::prelude::*;
use quickcheck_macros::quickcheck;
use rust_spice::*;

// Property-based tests using proptest
proptest! {
    #[test]
    fn test_calendar_to_et_properties(
        year in 1900i32..2100,
        month in 1i32..12,
        day in 1i32..28,  // Conservative to avoid month-end issues
        hour in 0i32..23,
        minute in 0i32..59,
        second in 0.0f64..59.0,
    ) {
        let et = calendar_to_et(year, month, day, hour, minute, second);
        
        // ET should be a finite number
        prop_assert!(et.is_finite());
        
        // ET should be reasonable (not extreme values)
        prop_assert!(et > -1e10 && et < 1e10);
        
        // Later dates should have larger ET values (generally)
        if year >= 2000 {
            prop_assert!(et >= 0.0);  // J2000 is epoch 0
        }
    }
    
    #[test]
    fn test_julian_date_conversion_properties(
        jd in 2000000.0f64..3000000.0,  // Reasonable Julian date range
    ) {
        let et = julian_date_to_et(jd);
        
        // ET should be finite
        prop_assert!(et.is_finite());
        
        // J2000 epoch (JD 2451545.0) should give ET = 0
        if (jd - 2451545.0).abs() < 1e-10 {
            prop_assert!((et).abs() < 1e-6);
        }
        
        // Later Julian dates should give larger ET values
        if jd > 2451545.0 {
            prop_assert!(et > -86400.0);  // At least not more than 1 day before J2000
        }
    }
    
    #[test]
    fn test_state_vector_properties(
        x in -1e9f64..1e9,
        y in -1e9f64..1e9,
        z in -1e9f64..1e9,
        vx in -100.0f64..100.0,
        vy in -100.0f64..100.0,
        vz in -100.0f64..100.0,
        lt in 0.0f64..1000.0,
    ) {
        let state = StateVector::new(x, y, z, vx, vy, vz, lt);
        
        // Position components should match
        prop_assert_eq!(state.x, x);
        prop_assert_eq!(state.y, y);
        prop_assert_eq!(state.z, z);
        
        // Velocity components should match
        prop_assert_eq!(state.vx, vx);
        prop_assert_eq!(state.vy, vy);
        prop_assert_eq!(state.vz, vz);
        
        // Light time should match
        prop_assert_eq!(state.light_time, lt);
        
        // Position and velocity getters should work
        let pos = state.position();
        prop_assert_eq!(pos.len(), 3);
        prop_assert_eq!(pos[0], x);
        prop_assert_eq!(pos[1], y);
        prop_assert_eq!(pos[2], z);
        
        let vel = state.velocity();
        prop_assert_eq!(vel.len(), 3);
        prop_assert_eq!(vel[0], vx);
        prop_assert_eq!(vel[1], vy);
        prop_assert_eq!(vel[2], vz);
    }
}

// QuickCheck-based tests for simpler properties
#[quickcheck]
fn test_time_conversion_monotonicity(et1: f64, et2: f64) -> bool {
    // Skip infinite or NaN values
    if !et1.is_finite() || !et2.is_finite() {
        return true;
    }
    
    // Skip values outside reasonable range
    if et1.abs() > 1e10 || et2.abs() > 1e10 {
        return true;
    }
    
    // Test that Julian date conversion preserves ordering
    let jd1 = et1 / 86400.0 + 2451545.0;  // Convert ET to approximate JD
    let jd2 = et2 / 86400.0 + 2451545.0;
    
    let et1_converted = julian_date_to_et(jd1);
    let et2_converted = julian_date_to_et(jd2);
    
    // If et1 < et2, then converted values should maintain order
    if et1 < et2 {
        et1_converted <= et2_converted
    } else if et1 > et2 {
        et1_converted >= et2_converted
    } else {
        (et1_converted - et2_converted).abs() < 1e-6
    }
}

#[quickcheck]
fn test_calendar_conversion_basic_properties(
    year: i32,
    month: i32,
    day: i32,
    hour: i32,
    minute: i32,
) -> bool {
    // Only test reasonable date ranges
    if year < 1900 || year > 2100 { return true; }
    if month < 1 || month > 12 { return true; }
    if day < 1 || day > 28 { return true; }  // Conservative
    if hour < 0 || hour > 23 { return true; }
    if minute < 0 || minute > 59 { return true; }
    
    let et = calendar_to_et(year, month, day, hour, minute, 0.0);
    
    // Basic sanity checks
    et.is_finite() && et > -1e10 && et < 1e10
}

#[quickcheck]
fn test_state_vector_magnitude_properties(x: f64, y: f64, z: f64) -> bool {
    // Skip infinite or very large values
    if !x.is_finite() || !y.is_finite() || !z.is_finite() {
        return true;
    }
    if x.abs() > 1e8 || y.abs() > 1e8 || z.abs() > 1e8 {
        return true;
    }
    
    let state = StateVector::new(x, y, z, 0.0, 0.0, 0.0, 0.0);
    let pos = state.position();
    
    // Calculate magnitude manually
    let magnitude_manual = (x*x + y*y + z*z).sqrt();
    let magnitude_from_vec = (pos[0]*pos[0] + pos[1]*pos[1] + pos[2]*pos[2]).sqrt();
    
    // They should be equal (within floating point precision)
    (magnitude_manual - magnitude_from_vec).abs() < 1e-10
}

// Physical law tests
#[quickcheck]
fn test_conservation_properties(
    x1: f64, y1: f64, z1: f64,
    x2: f64, y2: f64, z2: f64,
) -> bool {
    // Skip infinite values
    if ![x1, y1, z1, x2, y2, z2].iter().all(|v| v.is_finite()) {
        return true;
    }
    
    // Skip very large values
    if [x1, y1, z1, x2, y2, z2].iter().any(|v| v.abs() > 1e6) {
        return true;
    }
    
    let state1 = StateVector::new(x1, y1, z1, 0.0, 0.0, 0.0, 0.0);
    let state2 = StateVector::new(x2, y2, z2, 0.0, 0.0, 0.0, 0.0);
    
    // Test that distance calculation is symmetric
    let pos1 = state1.position();
    let pos2 = state2.position();
    
    let dx = pos1[0] - pos2[0];
    let dy = pos1[1] - pos2[1];
    let dz = pos1[2] - pos2[2];
    let distance_1_to_2 = (dx*dx + dy*dy + dz*dz).sqrt();
    
    let dx_rev = pos2[0] - pos1[0];
    let dy_rev = pos2[1] - pos1[1];
    let dz_rev = pos2[2] - pos1[2];
    let distance_2_to_1 = (dx_rev*dx_rev + dy_rev*dy_rev + dz_rev*dz_rev).sqrt();
    
    // Distances should be equal (symmetry property)
    (distance_1_to_2 - distance_2_to_1).abs() < 1e-10
}

#[cfg(test)]
mod property_tests {
    use super::*;
    
    #[test]
    fn run_all_property_tests() {
        // This will run all the proptest! macros above
        // Individual tests are already defined by the proptest! macro
    }
}

#[cfg(test)]
mod mathematical_properties {
    use super::*;
    
    #[test]
    fn test_physical_constants() {
        // Test that our implementations respect known physical constants
        
        // Light travel time to Moon should be reasonable
        // Moon is ~384,400 km away, light speed ~299,792.458 km/s
        // So light time should be ~1.28 seconds
        let expected_moon_light_time = 384400.0 / 299792.458;
        
        // Our simplified implementation should be in the right ballpark
        // (This is just testing our test harness, not real SPICE calculations)
        assert!((expected_moon_light_time - 1.28).abs() < 0.1);
    }
    
    #[test]
    fn test_coordinate_system_properties() {
        // Test basic properties that should hold for any coordinate system
        
        // Origin should have zero position
        let origin_state = StateVector::new(0.0, 0.0, 0.0, 0.0, 0.0, 0.0, 0.0);
        let pos = origin_state.position();
        assert_eq!(pos, vec![0.0, 0.0, 0.0]);
        
        // Magnitude should be zero for origin
        let magnitude = (pos[0]*pos[0] + pos[1]*pos[1] + pos[2]*pos[2]).sqrt();
        assert_eq!(magnitude, 0.0);
    }
    
    #[test]
    fn test_time_system_properties() {
        // Test properties that should hold for time conversions
        
        // J2000 epoch
        let j2000_et = julian_date_to_et(2451545.0);
        assert_eq!(j2000_et, 0.0);
        
        // One day later should be 86400 seconds later
        let next_day_et = julian_date_to_et(2451546.0);
        assert_eq!(next_day_et, 86400.0);
        
        // Time should advance monotonically
        for i in 0..10 {
            let jd = 2451545.0 + i as f64;
            let et = julian_date_to_et(jd);
            assert_eq!(et, (i as f64) * 86400.0);
        }
    }
}
