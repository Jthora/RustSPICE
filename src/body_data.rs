//! Celestial body data and ID management for RustSPICE
//! 
//! This module provides planetary constants, body name/ID mappings, and physical
//! properties of celestial bodies. It implements the equivalent functionality
//! of CSPICE bodvrd_c, bodn2c_c, bodc2n_c, and related functions.

#[cfg(not(feature = "std"))]
extern crate alloc;

#[cfg(not(feature = "std"))]
use alloc::{collections::BTreeMap, format, string::String, vec::Vec};

#[cfg(feature = "std")]
use std::{collections::BTreeMap, format, string::String, vec::Vec, sync::OnceLock};

use crate::foundation::{SpiceInt, SpiceDouble};
use crate::error_handling::{SpiceResult, SpiceError, SpiceErrorType};
use crate::kernel_pool;

// Standard NAIF body codes (from NAIF documentation)
pub const SOLAR_SYSTEM_BARYCENTER: SpiceInt = 0;
pub const MERCURY_BARYCENTER: SpiceInt = 1;
pub const VENUS_BARYCENTER: SpiceInt = 2;
pub const EARTH_MOON_BARYCENTER: SpiceInt = 3;
pub const MARS_BARYCENTER: SpiceInt = 4;
pub const JUPITER_BARYCENTER: SpiceInt = 5;
pub const SATURN_BARYCENTER: SpiceInt = 6;
pub const URANUS_BARYCENTER: SpiceInt = 7;
pub const NEPTUNE_BARYCENTER: SpiceInt = 8;
pub const PLUTO_BARYCENTER: SpiceInt = 9;
pub const SUN: SpiceInt = 10;

// Individual bodies
pub const MERCURY: SpiceInt = 199;
pub const VENUS: SpiceInt = 299;
pub const EARTH: SpiceInt = 399;
pub const MARS: SpiceInt = 499;
pub const JUPITER: SpiceInt = 599;
pub const SATURN: SpiceInt = 699;
pub const URANUS: SpiceInt = 799;
pub const NEPTUNE: SpiceInt = 899;
pub const PLUTO: SpiceInt = 999;

// Earth's moon
pub const MOON: SpiceInt = 301;

// Major moons
pub const IO: SpiceInt = 501;
pub const EUROPA: SpiceInt = 502;
pub const GANYMEDE: SpiceInt = 503;
pub const CALLISTO: SpiceInt = 504;

pub const MIMAS: SpiceInt = 601;
pub const ENCELADUS: SpiceInt = 602;
pub const TETHYS: SpiceInt = 603;
pub const DIONE: SpiceInt = 604;
pub const RHEA: SpiceInt = 605;
pub const TITAN: SpiceInt = 606;
pub const IAPETUS: SpiceInt = 608;

/// Physical constants for celestial bodies
#[derive(Debug, Clone)]
pub struct BodyConstants {
    /// Mean radius in kilometers
    pub radii: Vec<SpiceDouble>,
    /// Gravitational parameter (GM) in km^3/s^2
    pub gm: Option<SpiceDouble>,
    /// J2 gravitational harmonic coefficient
    pub j2: Option<SpiceDouble>,
    /// Pole right ascension in degrees (J2000)
    pub pole_ra: Option<SpiceDouble>,
    /// Pole declination in degrees (J2000)
    pub pole_dec: Option<SpiceDouble>,
    /// Prime meridian angle in degrees
    pub pm: Option<SpiceDouble>,
    /// Rotation rate in degrees per day
    pub rotation_rate: Option<SpiceDouble>,
}

impl Default for BodyConstants {
    fn default() -> Self {
        Self {
            radii: vec![1.0, 1.0, 1.0],
            gm: None,
            j2: None,
            pole_ra: None,
            pole_dec: None,
            pm: None,
            rotation_rate: None,
        }
    }
}

/// Get a reference to the built-in body name/code mapping
fn get_builtin_body_mapping() -> &'static BTreeMap<&'static str, SpiceInt> {
    #[cfg(feature = "std")]
    {
        use std::sync::OnceLock;
        static BODY_MAPPING: OnceLock<BTreeMap<&'static str, SpiceInt>> = OnceLock::new();
        
        BODY_MAPPING.get_or_init(|| {
            let mut map = BTreeMap::new();
            
            // Solar system bodies
            map.insert("SOLAR SYSTEM BARYCENTER", SOLAR_SYSTEM_BARYCENTER);
            map.insert("SSB", SOLAR_SYSTEM_BARYCENTER);
            map.insert("SUN", SUN);
            
            // Planets
            map.insert("MERCURY BARYCENTER", MERCURY_BARYCENTER);
            map.insert("VENUS BARYCENTER", VENUS_BARYCENTER);  
            map.insert("EARTH BARYCENTER", EARTH_MOON_BARYCENTER);
            map.insert("MARS BARYCENTER", MARS_BARYCENTER);
            map.insert("JUPITER BARYCENTER", JUPITER_BARYCENTER);
            map.insert("SATURN BARYCENTER", SATURN_BARYCENTER);
            map.insert("URANUS BARYCENTER", URANUS_BARYCENTER);
            map.insert("NEPTUNE BARYCENTER", NEPTUNE_BARYCENTER);
            map.insert("PLUTO BARYCENTER", PLUTO_BARYCENTER);
            
            map.insert("MERCURY", MERCURY);
            map.insert("VENUS", VENUS);
            map.insert("EARTH", EARTH);
            map.insert("MARS", MARS);
            map.insert("JUPITER", JUPITER);
            map.insert("SATURN", SATURN);
            map.insert("URANUS", URANUS);
            map.insert("NEPTUNE", NEPTUNE);
            map.insert("PLUTO", PLUTO);
            
            // Moon
            map.insert("MOON", MOON);
            map.insert("LUNA", MOON);
            
            // Jupiter moons
            map.insert("IO", IO);
            map.insert("EUROPA", EUROPA);
            map.insert("GANYMEDE", GANYMEDE);
            map.insert("CALLISTO", CALLISTO);
            
            // Saturn moons
            map.insert("MIMAS", MIMAS);
            map.insert("ENCELADUS", ENCELADUS);
            map.insert("TETHYS", TETHYS);
            map.insert("DIONE", DIONE);
            map.insert("RHEA", RHEA);
            map.insert("TITAN", TITAN);
            map.insert("IAPETUS", IAPETUS);
            
            map
        })
    }
    
    #[cfg(not(feature = "std"))]
    {
        // For no_std, use a const static map
        static BODY_MAPPING_ARRAY: &[(&str, SpiceInt)] = &[
            ("SOLAR SYSTEM BARYCENTER", SOLAR_SYSTEM_BARYCENTER),
            ("SSB", SOLAR_SYSTEM_BARYCENTER),
            ("SUN", SUN),
            ("MERCURY BARYCENTER", MERCURY_BARYCENTER),
            ("VENUS BARYCENTER", VENUS_BARYCENTER),
            ("EARTH BARYCENTER", EARTH_MOON_BARYCENTER),
            ("MARS BARYCENTER", MARS_BARYCENTER),
            ("JUPITER BARYCENTER", JUPITER_BARYCENTER),
            ("SATURN BARYCENTER", SATURN_BARYCENTER),
            ("URANUS BARYCENTER", URANUS_BARYCENTER),
            ("NEPTUNE BARYCENTER", NEPTUNE_BARYCENTER),
            ("PLUTO BARYCENTER", PLUTO_BARYCENTER),
            ("MERCURY", MERCURY),
            ("VENUS", VENUS),
            ("EARTH", EARTH),
            ("MARS", MARS),
            ("JUPITER", JUPITER),
            ("SATURN", SATURN),
            ("URANUS", URANUS),
            ("NEPTUNE", NEPTUNE),
            ("PLUTO", PLUTO),
            ("MOON", MOON),
            ("LUNA", MOON),
            ("IO", IO),
            ("EUROPA", EUROPA),
            ("GANYMEDE", GANYMEDE),
            ("CALLISTO", CALLISTO),
            ("MIMAS", MIMAS),
            ("ENCELADUS", ENCELADUS),
            ("TETHYS", TETHYS),
            ("DIONE", DIONE),
            ("RHEA", RHEA),
            ("TITAN", TITAN),
            ("IAPETUS", IAPETUS),
        ];
        
        static mut BODY_MAPPING_ONCE: Option<BTreeMap<&'static str, SpiceInt>> = None;
        
        unsafe {
            if BODY_MAPPING_ONCE.is_none() {
                let mut map = BTreeMap::new();
                for &(name, code) in BODY_MAPPING_ARRAY {
                    map.insert(name, code);
                }
                BODY_MAPPING_ONCE = Some(map);
            }
            BODY_MAPPING_ONCE.as_ref().unwrap()
        }
    }
}

/// Get a reference to the built-in physical constants
fn get_builtin_constants() -> &'static BTreeMap<SpiceInt, BodyConstants> {
    use std::sync::OnceLock;
    static CONSTANTS: OnceLock<BTreeMap<SpiceInt, BodyConstants>> = OnceLock::new();
    
    CONSTANTS.get_or_init(|| {
        let mut map = BTreeMap::new();
        
        // Sun
        map.insert(SUN, BodyConstants {
            radii: vec![696000.0, 696000.0, 696000.0],
            gm: Some(1.32712440041e11),
            j2: None,
            pole_ra: Some(286.13),
            pole_dec: Some(63.87),
            pm: Some(84.176),
            rotation_rate: Some(14.18440),
        });
        
        // Earth
        map.insert(EARTH, BodyConstants {
            radii: vec![6378.1366, 6378.1366, 6356.7519],
            gm: Some(3.9860043543609598e5),
            j2: Some(1.0826359e-3),
            pole_ra: Some(0.0),
            pole_dec: Some(90.0),
            pm: Some(190.147),
            rotation_rate: Some(360.9856235),
        });
        
        // Moon
        map.insert(MOON, BodyConstants {
            radii: vec![1737.4, 1737.4, 1737.4],
            gm: Some(4.9028695e3),
            j2: Some(2.027e-4),
            pole_ra: Some(269.9949),
            pole_dec: Some(66.5392),
            pm: Some(38.3213),
            rotation_rate: Some(13.17635815),
        });
        
        // Mars
        map.insert(MARS, BodyConstants {
            radii: vec![3396.19, 3396.19, 3376.20],
            gm: Some(4.282837e4),
            j2: Some(1.956e-3),
            pole_ra: Some(317.68143),
            pole_dec: Some(52.8865),
            pm: Some(176.630),
            rotation_rate: Some(350.89198226),
        });
        
        // Jupiter
        map.insert(JUPITER, BodyConstants {
            radii: vec![71492.0, 71492.0, 66854.0],
            gm: Some(1.266865349e8),
            j2: Some(1.4697e-2),
            pole_ra: Some(268.056595),
            pole_dec: Some(64.495303),
            pm: Some(284.95),
            rotation_rate: Some(870.5360000),
        });
        
        // Add more bodies as needed...
        map
    })
}

/// Convert body name to NAIF ID (equivalent to bodn2c_c)
/// 
/// # Arguments
/// * `name` - Body name (case insensitive)
/// 
/// # Returns
/// * `Ok(code)` - NAIF ID code for the body
/// * `Err(SpiceError)` - If body name is not recognized
pub fn body_name_to_code(name: &str) -> SpiceResult<SpiceInt> {
    let upper_name = name.to_uppercase();
    
    // First check built-in mapping
    if let Some(&code) = get_builtin_body_mapping().get(upper_name.as_str()) {
        return Ok(code);
    }
    
    // Check kernel pool for custom body definitions
    let pool_key = format!("NAIF_BODY_NAME_{}", upper_name);
    if kernel_pool::exists_in_pool(&pool_key).unwrap_or(false) {
        let (codes, _) = kernel_pool::get_integer_pool(&pool_key, 0, 1)?;
        if !codes.is_empty() {
            return Ok(codes[0]);
        }
    }
    
    // Try parsing as integer
    if let Ok(code) = name.parse::<SpiceInt>() {
        return Ok(code);
    }
    
    Err(SpiceError::new(
        SpiceErrorType::InvalidTarget,
        format!("Body name '{}' not recognized", name)
    ))
}

/// Convert NAIF ID to body name (equivalent to bodc2n_c)
/// 
/// # Arguments
/// * `code` - NAIF ID code
/// 
/// # Returns
/// * `Ok(name)` - Body name corresponding to the code
/// * `Err(SpiceError)` - If body code is not recognized
pub fn body_code_to_name(code: SpiceInt) -> SpiceResult<String> {
    // Define preferred names for bodies with multiple aliases
    let preferred_name = match code {
        MOON => "MOON",
        EARTH => "EARTH", 
        SUN => "SUN",
        MARS => "MARS",
        JUPITER => "JUPITER",
        SATURN => "SATURN",
        URANUS => "URANUS",
        NEPTUNE => "NEPTUNE",
        PLUTO => "PLUTO",
        MERCURY => "MERCURY",
        VENUS => "VENUS",
        _ => "",
    };
    
    // Return preferred name if available
    if !preferred_name.is_empty() {
        return Ok(preferred_name.to_string());
    }
    
    // Search built-in mapping for other bodies
    for (&name, &id) in get_builtin_body_mapping().iter() {
        if id == code {
            return Ok(name.to_string());
        }
    }
    
    // Check kernel pool for custom body definitions
    let pool_key = format!("NAIF_BODY_CODE_{}", code);
    if kernel_pool::exists_in_pool(&pool_key).unwrap_or(false) {
        let (names, _) = kernel_pool::get_character_pool(&pool_key, 0, 1)?;
        if !names.is_empty() {
            return Ok(names[0].clone());
        }
    }
    
    Err(SpiceError::new(
        SpiceErrorType::InvalidTarget,
        format!("Body code {} not recognized", code)
    ))
}

/// Retrieve body constants from kernel pool or built-in data (equivalent to bodvrd_c)
/// 
/// # Arguments
/// * `body` - Body name or NAIF ID
/// * `item` - Constant name (e.g., "RADII", "GM", "J2")
/// 
/// # Returns
/// * `Ok(values)` - Vector of constant values
/// * `Err(SpiceError)` - If body or constant not found
pub fn body_data(body: &str, item: &str) -> SpiceResult<Vec<SpiceDouble>> {
    // Convert body name to code if needed
    let code = if let Ok(c) = body.parse::<SpiceInt>() {
        c
    } else {
        body_name_to_code(body)?
    };
    
    let item_upper = item.to_uppercase();
    
    // First check kernel pool for the specific body and item
    let pool_key = format!("BODY{}_{}",code, item_upper);
    if kernel_pool::exists_in_pool(&pool_key).unwrap_or(false) {
        let (values, _) = kernel_pool::get_double_pool(&pool_key, 0, 10)?;
        if !values.is_empty() {
            return Ok(values);
        }
    }
    
    // Check built-in constants
    if let Some(constants) = get_builtin_constants().get(&code) {
        match item_upper.as_str() {
            "RADII" => return Ok(constants.radii.clone()),
            "GM" => {
                if let Some(gm) = constants.gm {
                    return Ok(vec![gm]);
                }
            },
            "J2" => {
                if let Some(j2) = constants.j2 {
                    return Ok(vec![j2]);
                }
            },
            "POLE_RA" => {
                if let Some(pole_ra) = constants.pole_ra {
                    return Ok(vec![pole_ra]);
                }
            },
            "POLE_DEC" => {
                if let Some(pole_dec) = constants.pole_dec {
                    return Ok(vec![pole_dec]);
                }
            },
            "PM" => {
                if let Some(pm) = constants.pm {
                    return Ok(vec![pm]);
                }
            },
            "NUT_PREC_RA" | "NUT_PREC_DEC" | "NUT_PREC_PM" => {
                // Return zeros for nutation/precession coefficients if not available
                return Ok(vec![0.0]);
            },
            _ => {}
        }
    }
    
    Err(SpiceError::new(
        SpiceErrorType::InsufficientData,
        format!("Body data item '{}' not found for body '{}'", item, body)
    ))
}

/// Check if a body is recognized (equivalent to bodfnd_c)
/// 
/// # Arguments
/// * `body` - Body name or NAIF ID
/// 
/// # Returns
/// * `true` if body is recognized, `false` otherwise
pub fn body_found(body: &str) -> bool {
    let upper_name = body.to_uppercase();
    
    // Check built-in name mapping first
    if get_builtin_body_mapping().contains_key(upper_name.as_str()) {
        return true;
    }
    
    // Check kernel pool for custom body definitions by name
    let pool_key = format!("NAIF_BODY_NAME_{}", upper_name);
    if kernel_pool::exists_in_pool(&pool_key).unwrap_or(false) {
        return true;
    }
    
    // Try parsing as integer and check if the code has a known name
    if let Ok(code) = body.parse::<SpiceInt>() {
        // Only accept numeric codes that have corresponding names or kernel pool entries
        if body_code_to_name(code).is_ok() {
            return true;
        }
        
        // Check if the code exists in kernel pool
        let pool_key = format!("NAIF_BODY_CODE_{}", code);
        if kernel_pool::exists_in_pool(&pool_key).unwrap_or(false) {
            return true;
        }
    }
    
    false
}

/// Get the central body for a given body (equivalent to ccifrm_c logic)
/// 
/// # Arguments
/// * `body` - Body name or NAIF ID
/// 
/// # Returns
/// * `Ok(center)` - NAIF ID of the central body
/// * `Err(SpiceError)` - If body not found or center indeterminate
pub fn body_center(body: &str) -> SpiceResult<SpiceInt> {
    let code = if let Ok(c) = body.parse::<SpiceInt>() {
        c
    } else {
        body_name_to_code(body)?
    };
    
    // Determine center based on NAIF ID conventions
    match code {
        0 => Err(SpiceError::new(
            SpiceErrorType::InvalidTarget,
            "Solar system barycenter has no center".into()
        )),
        10 => Ok(SOLAR_SYSTEM_BARYCENTER), // Sun orbits SSB
        1..=9 => Ok(SOLAR_SYSTEM_BARYCENTER), // Planet barycenters orbit SSB
        199 => Ok(MERCURY_BARYCENTER),
        299 => Ok(VENUS_BARYCENTER),
        301..=398 => Ok(EARTH_MOON_BARYCENTER), // Earth-Moon system moons
        399 => Ok(EARTH_MOON_BARYCENTER), // Earth itself
        401..=498 => Ok(MARS_BARYCENTER), // Mars system moons
        499 => Ok(MARS_BARYCENTER), // Mars itself
        501..=598 => Ok(JUPITER_BARYCENTER), // Jupiter system moons
        599 => Ok(JUPITER_BARYCENTER), // Jupiter itself
        601..=698 => Ok(SATURN_BARYCENTER), // Saturn system moons
        699 => Ok(SATURN_BARYCENTER), // Saturn itself
        701..=798 => Ok(URANUS_BARYCENTER), // Uranus system moons
        799 => Ok(URANUS_BARYCENTER), // Uranus itself
        801..=898 => Ok(NEPTUNE_BARYCENTER), // Neptune system moons
        899 => Ok(NEPTUNE_BARYCENTER), // Neptune itself
        901..=998 => Ok(PLUTO_BARYCENTER), // Pluto system moons
        999 => Ok(PLUTO_BARYCENTER), // Pluto itself
        _ => {
            // Check kernel pool for custom definitions
            let pool_key = format!("BODY{}_CENTER", code);
            if kernel_pool::exists_in_pool(&pool_key).unwrap_or(false) {
                let (centers, _) = kernel_pool::get_integer_pool(&pool_key, 0, 1)?;
                if !centers.is_empty() {
                    return Ok(centers[0]);
                }
            }
            
            Ok(SOLAR_SYSTEM_BARYCENTER) // Default to SSB
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::kernel_system::initialize_kernel_system;
    use crate::kernel_pool::initialize_pool;

    #[test]
    fn test_body_name_to_code() {
        // Test built-in body names
        assert_eq!(body_name_to_code("EARTH").unwrap(), EARTH);
        assert_eq!(body_name_to_code("earth").unwrap(), EARTH); // Case insensitive
        assert_eq!(body_name_to_code("MARS").unwrap(), MARS);
        assert_eq!(body_name_to_code("MOON").unwrap(), MOON);
        assert_eq!(body_name_to_code("LUNA").unwrap(), MOON); // Alternative name
        assert_eq!(body_name_to_code("SUN").unwrap(), SUN);
        assert_eq!(body_name_to_code("SSB").unwrap(), SOLAR_SYSTEM_BARYCENTER);
        
        // Test numeric input
        assert_eq!(body_name_to_code("399").unwrap(), EARTH);
        assert_eq!(body_name_to_code("499").unwrap(), MARS);
        
        // Test Jupiter moons
        assert_eq!(body_name_to_code("IO").unwrap(), IO);
        assert_eq!(body_name_to_code("EUROPA").unwrap(), EUROPA);
        assert_eq!(body_name_to_code("GANYMEDE").unwrap(), GANYMEDE);
        assert_eq!(body_name_to_code("CALLISTO").unwrap(), CALLISTO);
        
        // Test Saturn moons
        assert_eq!(body_name_to_code("TITAN").unwrap(), TITAN);
        assert_eq!(body_name_to_code("ENCELADUS").unwrap(), ENCELADUS);
        
        // Test invalid body name
        assert!(body_name_to_code("INVALID_BODY").is_err());
    }

    #[test]
    fn test_body_code_to_name() {
        // Test built-in body codes
        assert_eq!(body_code_to_name(EARTH).unwrap(), "EARTH");
        assert_eq!(body_code_to_name(MARS).unwrap(), "MARS");
        assert_eq!(body_code_to_name(MOON).unwrap(), "MOON");
        assert_eq!(body_code_to_name(SUN).unwrap(), "SUN");
        assert_eq!(body_code_to_name(SOLAR_SYSTEM_BARYCENTER).unwrap(), "SOLAR SYSTEM BARYCENTER");
        
        // Test Jupiter moons
        assert_eq!(body_code_to_name(IO).unwrap(), "IO");
        assert_eq!(body_code_to_name(EUROPA).unwrap(), "EUROPA");
        
        // Test Saturn moons
        assert_eq!(body_code_to_name(TITAN).unwrap(), "TITAN");
        
        // Test invalid body code
        assert!(body_code_to_name(99999).is_err());
    }

    #[test]
    fn test_body_data() {
        // Test Earth constants
        let earth_radii = body_data("EARTH", "RADII").unwrap();
        assert_eq!(earth_radii.len(), 3);
        assert!((earth_radii[0] - 6378.1366).abs() < 1e-4);
        assert!((earth_radii[2] - 6356.7519).abs() < 1e-4);
        
        let earth_gm = body_data("EARTH", "GM").unwrap();
        assert_eq!(earth_gm.len(), 1);
        assert!((earth_gm[0] - 3.9860043543609598e5).abs() < 1e5);
        
        // Test Moon constants
        let moon_radii = body_data("MOON", "RADII").unwrap();
        assert_eq!(moon_radii.len(), 3);
        assert!((moon_radii[0] - 1737.4).abs() < 0.1);
        
        let moon_gm = body_data("MOON", "GM").unwrap();
        assert_eq!(moon_gm.len(), 1);
        assert!((moon_gm[0] - 4.9028695e3).abs() < 1e2);
        
        // Test Mars constants
        let mars_radii = body_data("MARS", "RADII").unwrap();
        assert_eq!(mars_radii.len(), 3);
        assert!((mars_radii[0] - 3396.19).abs() < 0.1);
        
        // Test numeric body ID
        let earth_radii_by_id = body_data("399", "RADII").unwrap();
        assert_eq!(earth_radii_by_id, earth_radii);
        
        // Test case insensitivity
        let earth_radii_lower = body_data("earth", "radii").unwrap();
        assert_eq!(earth_radii_lower, earth_radii);
        
        // Test invalid body
        assert!(body_data("INVALID_BODY", "RADII").is_err());
        
        // Test invalid item
        assert!(body_data("EARTH", "INVALID_ITEM").is_err());
    }

    #[test]
    fn test_body_found() {
        // Test valid bodies
        assert!(body_found("EARTH"));
        assert!(body_found("earth")); // Case insensitive
        assert!(body_found("MARS"));
        assert!(body_found("MOON"));
        assert!(body_found("SUN"));
        assert!(body_found("399")); // Numeric
        
        // Test Jupiter and Saturn moons
        assert!(body_found("IO"));
        assert!(body_found("EUROPA"));
        assert!(body_found("TITAN"));
        assert!(body_found("ENCELADUS"));
        
        // Test invalid body
        assert!(!body_found("INVALID_BODY"));
        assert!(!body_found("99999"));
    }

    #[test]
    fn test_body_center() {
        // Test planetary bodies
        assert_eq!(body_center("EARTH").unwrap(), EARTH_MOON_BARYCENTER);
        assert_eq!(body_center("MARS").unwrap(), MARS_BARYCENTER);
        assert_eq!(body_center("JUPITER").unwrap(), JUPITER_BARYCENTER);
        
        // Test moons
        assert_eq!(body_center("MOON").unwrap(), EARTH_MOON_BARYCENTER);
        assert_eq!(body_center("IO").unwrap(), JUPITER_BARYCENTER);
        assert_eq!(body_center("EUROPA").unwrap(), JUPITER_BARYCENTER);
        assert_eq!(body_center("TITAN").unwrap(), SATURN_BARYCENTER);
        
        // Test planet barycenters
        assert_eq!(body_center("EARTH BARYCENTER").unwrap(), SOLAR_SYSTEM_BARYCENTER);
        assert_eq!(body_center("MARS BARYCENTER").unwrap(), SOLAR_SYSTEM_BARYCENTER);
        
        // Test Sun
        assert_eq!(body_center("SUN").unwrap(), SOLAR_SYSTEM_BARYCENTER);
        
        // Test numeric IDs
        assert_eq!(body_center("399").unwrap(), EARTH_MOON_BARYCENTER);
        assert_eq!(body_center("301").unwrap(), EARTH_MOON_BARYCENTER);
        
        // Test SSB (should error)
        assert!(body_center("SOLAR SYSTEM BARYCENTER").is_err());
        assert!(body_center("0").is_err());
    }

    #[test]
    fn test_physical_constants_accuracy() {
        // Test that our constants are within reasonable ranges
        
        // Earth
        let earth_gm = body_data("EARTH", "GM").unwrap()[0];
        assert!(earth_gm > 3.98e5 && earth_gm < 3.99e5); // Earth GM in km^3/s^2
        
        let earth_j2 = body_data("EARTH", "J2").unwrap()[0];
        assert!(earth_j2 > 1.08e-3 && earth_j2 < 1.09e-3); // Earth J2
        
        // Moon
        let moon_gm = body_data("MOON", "GM").unwrap()[0];
        assert!(moon_gm > 4.9e3 && moon_gm < 4.91e3); // Moon GM in km^3/s^2
        
        // Mars
        let mars_gm = body_data("MARS", "GM").unwrap()[0];
        assert!(mars_gm > 4.28e4 && mars_gm < 4.29e4); // Mars GM in km^3/s^2
        
        // Jupiter
        let jupiter_gm = body_data("JUPITER", "GM").unwrap()[0];
        assert!(jupiter_gm > 1.26e8 && jupiter_gm < 1.27e8); // Jupiter GM in km^3/s^2
    }

    #[test]
    fn test_kernel_pool_integration() {
        initialize_kernel_system().unwrap();
        initialize_pool().unwrap();
        
        // The body data functions should work without requiring kernel pool data
        // but should be able to use kernel pool data when available
        assert!(body_name_to_code("EARTH").is_ok());
        assert!(body_data("EARTH", "RADII").is_ok());
    }

    #[test]
    fn test_body_name_aliases() {
        // Test that alternative names work
        assert_eq!(body_name_to_code("LUNA").unwrap(), MOON);
        assert_eq!(body_name_to_code("MOON").unwrap(), MOON);
        
        // Both should map to the same code
        assert_eq!(body_name_to_code("LUNA").unwrap(), body_name_to_code("MOON").unwrap());
    }

    #[test]
    fn test_roundtrip_conversion() {
        // Test that name -> code -> name gives consistent results
        let bodies = ["EARTH", "MARS", "MOON", "SUN", "JUPITER", "IO", "TITAN"];
        
        for &body in &bodies {
            let code = body_name_to_code(body).unwrap();
            let name_back = body_code_to_name(code).unwrap();
            
            // The name might not be exactly the same (e.g., "LUNA" -> "MOON")
            // but converting back should give a valid code
            assert!(body_name_to_code(&name_back).is_ok());
        }
    }
}
