//! WASM bindings for RustSPICE time system functions
//! 
//! This module provides WebAssembly bindings for time conversion functions,
//! making them accessible from JavaScript/TypeScript.

#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[cfg(target_arch = "wasm32")]
use crate::foundation::EphemerisTime;
#[cfg(target_arch = "wasm32")]
use crate::time_system::{str_to_et, et_to_utc, time_parse, time_output};

/// WASM-compatible time result
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
#[derive(Clone)]
pub struct TimeResult {
    success: bool,
    value: f64,
    error_message: String,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl TimeResult {
    #[wasm_bindgen(getter)]
    pub fn success(&self) -> bool {
        self.success
    }

    #[wasm_bindgen(getter)]
    pub fn value(&self) -> f64 {
        self.value
    }

    #[wasm_bindgen(getter)]
    pub fn error_message(&self) -> String {
        self.error_message.clone()
    }
}

/// WASM-compatible string result
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
#[derive(Clone)]
pub struct StringResult {
    success: bool,
    value: String,
    error_message: String,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl StringResult {
    #[wasm_bindgen(getter)]
    pub fn success(&self) -> bool {
        self.success
    }

    #[wasm_bindgen(getter)]
    pub fn value(&self) -> String {
        self.value.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn error_message(&self) -> String {
        self.error_message.clone()
    }
}

/// WASM-compatible parsed time structure
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
#[derive(Clone)]
pub struct WasmParsedTime {
    year: i32,
    month: i32,
    day: i32,
    hour: i32,
    minute: i32,
    second: f64,
    day_of_year: i32,
    julian_date: f64,
    is_utc: bool,
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
impl WasmParsedTime {
    #[wasm_bindgen(getter)]
    pub fn year(&self) -> i32 { self.year }

    #[wasm_bindgen(getter)]
    pub fn month(&self) -> i32 { self.month }

    #[wasm_bindgen(getter)]
    pub fn day(&self) -> i32 { self.day }

    #[wasm_bindgen(getter)]
    pub fn hour(&self) -> i32 { self.hour }

    #[wasm_bindgen(getter)]
    pub fn minute(&self) -> i32 { self.minute }

    #[wasm_bindgen(getter)]
    pub fn second(&self) -> f64 { self.second }

    #[wasm_bindgen(getter)]
    pub fn day_of_year(&self) -> i32 { self.day_of_year }

    #[wasm_bindgen(getter)]
    pub fn julian_date(&self) -> f64 { self.julian_date }

    #[wasm_bindgen(getter)]
    pub fn is_utc(&self) -> bool { self.is_utc }
}

/// Convert time string to ephemeris time (WASM version of str2et_c)
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn wasm_str_to_et(time_string: &str) -> TimeResult {
    match str_to_et(time_string) {
        Ok(et) => TimeResult {
            success: true,
            value: et.seconds(),
            error_message: String::new(),
        },
        Err(e) => TimeResult {
            success: false,
            value: 0.0,
            error_message: format!("{}", e),
        },
    }
}

/// Convert ephemeris time to UTC string (WASM version of et2utc_c)
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn wasm_et_to_utc(et_seconds: f64, format: &str, precision: i32) -> StringResult {
    let et = EphemerisTime::new(et_seconds);
    match et_to_utc(et, format, precision) {
        Ok(utc_string) => StringResult {
            success: true,
            value: utc_string,
            error_message: String::new(),
        },
        Err(e) => StringResult {
            success: false,
            value: String::new(),
            error_message: format!("{}", e),
        },
    }
}

/// Parse time string and return detailed components (WASM version of tparse_c)
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn wasm_time_parse(time_string: &str) -> WasmParsedTime {
    match time_parse(time_string) {
        Ok(parsed) => WasmParsedTime {
            year: parsed.year,
            month: parsed.month,
            day: parsed.day,
            hour: parsed.hour,
            minute: parsed.minute,
            second: parsed.second,
            day_of_year: parsed.day_of_year,
            julian_date: parsed.julian_date,
            is_utc: true, // Assume UTC for now
        },
        Err(_) => WasmParsedTime {
            year: -1,
            month: -1,
            day: -1,
            hour: -1,
            minute: -1,
            second: -1.0,
            day_of_year: -1,
            julian_date: -1.0,
            is_utc: false,
        },
    }
}

/// Format time with custom picture string (WASM version of timout_c)
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn wasm_time_output(et_seconds: f64, picture: &str) -> StringResult {
    let et = EphemerisTime::new(et_seconds);
    match time_output(et, picture) {
        Ok(formatted) => StringResult {
            success: true,
            value: formatted,
            error_message: String::new(),
        },
        Err(e) => StringResult {
            success: false,
            value: String::new(),
            error_message: format!("{}", e),
        },
    }
}

/// Get current J2000 epoch time in seconds (useful for testing)
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn wasm_j2000_epoch() -> f64 {
    EphemerisTime::j2000().seconds()
}

/// Check if a year is a leap year (utility function)
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn wasm_is_leap_year(year: i32) -> bool {
    if year % 4 != 0 {
        false
    } else if year % 100 != 0 {
        true
    } else if year % 400 != 0 {
        false
    } else {
        true
    }
}

/// Convert Julian Date to Ephemeris Time
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn wasm_julian_date_to_et(julian_date: f64) -> f64 {
    let jd = crate::foundation::JulianDate::new(julian_date);
    jd.to_ephemeris_time().seconds()
}

/// Convert Ephemeris Time to Julian Date
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn wasm_et_to_julian_date(et_seconds: f64) -> f64 {
    let et = EphemerisTime::new(et_seconds);
    let jd_days = (et.seconds() / 86400.0) + 2451545.0;
    jd_days
}

/// Validate that a time string can be parsed (returns true if valid)
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn wasm_validate_time_string(time_string: &str) -> bool {
    str_to_et(time_string).is_ok()
}

/// Get the difference between ET and UTC (approximate leap seconds)
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn wasm_delta_et_utc(et_seconds: f64) -> f64 {
    let et = EphemerisTime::new(et_seconds);
    crate::time_system::delta_et_utc(et).unwrap_or(64.184)
}

// Console logging for WASM debugging
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

#[cfg(target_arch = "wasm32")]
macro_rules! console_log {
    ($($t:tt)*) => (log(&format!($($t)*)))
}

/// Initialize WASM module and log successful loading
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen(start)]
pub fn wasm_initialize() {
    console_log!("RustSPICE WASM module loaded successfully!");
    console_log!("Time system functions available:");
    console_log!("- wasm_str_to_et(timeString)");
    console_log!("- wasm_et_to_utc(etSeconds, format, precision)");
    console_log!("- wasm_time_parse(timeString)");
    console_log!("- wasm_time_output(etSeconds, picture)");
}
