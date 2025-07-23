use insta::{assert_debug_snapshot, assert_json_snapshot};
use serde_json::json;
use rust_spice::*;

#[cfg(test)]
mod snapshot_tests {
    use super::*;
    
    #[test]
    fn test_state_vector_creation_snapshot() {
        // Test standard planetary positions (simplified test data)
        let earth_state = StateVector::new(
            149597870.7,  // 1 AU in km
            0.0,
            0.0,
            0.0,
            29.78,  // Earth orbital velocity km/s
            0.0,
            0.0
        );
        
        assert_debug_snapshot!(earth_state);
    }
    
    #[test]
    fn test_time_conversion_snapshots() {
        // Test known time conversions
        let test_times = vec![
            ("J2000 Epoch", 2451545.0),
            ("One day after J2000", 2451546.0),
            ("One year after J2000", 2451910.0),  // Approximate
            ("Unix Epoch", 2440587.5),  // Jan 1, 1970
            ("Y2K", 2451544.5),  // Dec 31, 1999 12:00 UTC
        ];
        
        let mut results = Vec::new();
        for (name, julian_date) in test_times {
            let et = julian_date_to_et(julian_date);
            results.push(json!({
                "description": name,
                "julian_date": julian_date,
                "ephemeris_time": et,
                "days_since_j2000": (julian_date - 2451545.0),
                "seconds_since_j2000": et
            }));
        }
        
        assert_json_snapshot!(results);
    }
    
    #[test]
    fn test_calendar_conversion_snapshots() {
        // Test specific calendar dates
        let test_dates = vec![
            ("J2000 Epoch", 2000, 1, 1, 12, 0, 0.0),
            ("Apollo 11 Launch", 1969, 7, 16, 13, 32, 0.0),
            ("Galileo Launch", 1989, 10, 18, 16, 53, 40.0),
            ("Cassini Launch", 1997, 10, 15, 8, 43, 0.0),
            ("New Horizons Launch", 2006, 1, 19, 19, 0, 0.0),
        ];
        
        let mut results = Vec::new();
        for (name, year, month, day, hour, minute, second) in test_dates {
            let et = calendar_to_et(year, month, day, hour, minute, second);
            results.push(json!({
                "description": name,
                "date": format!("{:04}-{:02}-{:02}T{:02}:{:02}:{:06.3}", 
                    year, month, day, hour, minute, second),
                "ephemeris_time": et,
                "year": year,
                "month": month,
                "day": day,
                "hour": hour,
                "minute": minute,
                "second": second
            }));
        }
        
        assert_json_snapshot!(results);
    }
    
    #[test]
    fn test_error_handling_snapshots() {
        // Test various error conditions
        let errors = vec![
            SpiceError::KernelNotFound,
            SpiceError::InvalidTime,
            SpiceError::InvalidTarget,
            SpiceError::ComputationError("Test computation failed".to_string()),
        ];
        
        assert_debug_snapshot!(errors);
    }
    
    #[test]
    fn test_planetary_positions_snapshot() {
        // Test simplified planetary position calculations
        // (These are test values, not real ephemeris data)
        let planets = vec![
            ("Mercury", StateVector::new(57.9e6, 0.0, 0.0, 0.0, 47.9, 0.0, 0.0)),
            ("Venus", StateVector::new(108.2e6, 0.0, 0.0, 0.0, 35.0, 0.0, 0.0)),
            ("Earth", StateVector::new(149.6e6, 0.0, 0.0, 0.0, 29.8, 0.0, 0.0)),
            ("Mars", StateVector::new(227.9e6, 0.0, 0.0, 0.0, 24.1, 0.0, 0.0)),
            ("Jupiter", StateVector::new(778.5e6, 0.0, 0.0, 0.0, 13.1, 0.0, 0.0)),
        ];
        
        let mut planetary_data = Vec::new();
        for (name, state) in planets {
            let pos = state.position();
            let vel = state.velocity();
            let distance_au = (pos[0] * pos[0] + pos[1] * pos[1] + pos[2] * pos[2]).sqrt() / 149597870.7;
            let speed = (vel[0] * vel[0] + vel[1] * vel[1] + vel[2] * vel[2]).sqrt();
            
            planetary_data.push(json!({
                "name": name,
                "position_km": pos,
                "velocity_km_s": vel,
                "distance_au": distance_au,
                "orbital_speed_km_s": speed,
                "light_time_seconds": state.light_time
            }));
        }
        
        assert_json_snapshot!(planetary_data);
    }
    
    #[test]
    fn test_coordinate_transformations_snapshot() {
        // Test coordinate system transformations
        let test_points = vec![
            ("Origin", 0.0, 0.0, 0.0),
            ("X-axis", 1.0, 0.0, 0.0),
            ("Y-axis", 0.0, 1.0, 0.0),
            ("Z-axis", 0.0, 0.0, 1.0),
            ("Diagonal", 1.0, 1.0, 1.0),
            ("Earth position", 6378.137, 0.0, 0.0),  // Earth radius in km
        ];
        
        let mut coordinate_data = Vec::new();
        for (name, x, y, z) in test_points {
            let state = StateVector::new(x, y, z, 0.0, 0.0, 0.0, 0.0);
            let pos = state.position();
            let magnitude = (x*x + y*y + z*z).sqrt();
            
            // Convert to spherical coordinates
            let r = magnitude;
            let theta = if r > 0.0 { (z / r).acos() } else { 0.0 };  // Colatitude
            let phi = y.atan2(x);  // Azimuth
            
            coordinate_data.push(json!({
                "description": name,
                "cartesian": {
                    "x": x,
                    "y": y,
                    "z": z
                },
                "spherical": {
                    "radius": r,
                    "colatitude_rad": theta,
                    "azimuth_rad": phi,
                    "colatitude_deg": theta.to_degrees(),
                    "azimuth_deg": phi.to_degrees()
                },
                "magnitude": magnitude
            }));
        }
        
        assert_json_snapshot!(coordinate_data);
    }
    
    #[test]
    fn test_physics_constants_snapshot() {
        // Test that our physical constants are reasonable
        let constants = json!({
            "speed_of_light_km_s": 299792.458,
            "au_km": 149597870.7,
            "earth_radius_km": 6378.137,
            "earth_orbital_velocity_km_s": 29.78,
            "seconds_per_day": 86400,
            "j2000_julian_date": 2451545.0,
            "unix_epoch_julian_date": 2440587.5
        });
        
        assert_json_snapshot!(constants);
    }
    
    #[test]
    fn test_typical_mission_scenarios_snapshot() {
        // Test scenarios typical of space missions
        let scenarios = vec![
            json!({
                "mission": "Earth-Moon Transfer",
                "initial_state": StateVector::new(6578.0, 0.0, 0.0, 0.0, 11.0, 0.0, 0.0),
                "description": "Low Earth orbit departure for lunar transfer"
            }),
            json!({
                "mission": "Mars Approach",
                "initial_state": StateVector::new(227.9e6, 0.0, 0.0, 0.0, 24.1, 0.0, 900.0),
                "description": "Spacecraft approaching Mars with 15-minute light delay"
            }),
            json!({
                "mission": "Jupiter Flyby",
                "initial_state": StateVector::new(778.5e6, 0.0, 0.0, 0.0, 13.1, 0.0, 2600.0),
                "description": "Deep space probe at Jupiter distance"
            }),
        ];
        
        // Add calculated properties to each scenario
        for scenario in &mut scenarios.iter_mut() {
            if let Some(state_obj) = scenario.get("initial_state") {
                // This is a simplified calculation for snapshot testing
                let pos = vec![
                    state_obj.get("x").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    state_obj.get("y").and_then(|v| v.as_f64()).unwrap_or(0.0),
                    state_obj.get("z").and_then(|v| v.as_f64()).unwrap_or(0.0),
                ];
                let distance = (pos[0]*pos[0] + pos[1]*pos[1] + pos[2]*pos[2]).sqrt();
                
                scenario.as_object_mut().unwrap().insert(
                    "calculated_distance_km".to_string(),
                    json!(distance)
                );
                scenario.as_object_mut().unwrap().insert(
                    "distance_au".to_string(),
                    json!(distance / 149597870.7)
                );
            }
        }
        
        assert_json_snapshot!(scenarios);
    }
}
