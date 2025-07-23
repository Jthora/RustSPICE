use wasm_bindgen::prelude::*;
use serde::{Serialize, Deserialize};

// Import console.log for debugging
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

macro_rules! console_log {
    ($($t:tt)*) => (log(&format_args!($($t)*).to_string()))
}

#[wasm_bindgen(start)]
pub fn main() {
    console_log!("RustSPICE WASM module initialized");
}

/// Primary data structure for position and velocity
#[wasm_bindgen]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateVector {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub vx: f64,
    pub vy: f64,
    pub vz: f64,
    pub light_time: f64,
}

#[wasm_bindgen]
impl StateVector {
    #[wasm_bindgen(constructor)]
    pub fn new(x: f64, y: f64, z: f64, vx: f64, vy: f64, vz: f64, light_time: f64) -> StateVector {
        StateVector { x, y, z, vx, vy, vz, light_time }
    }
    
    #[wasm_bindgen]
    pub fn position(&self) -> Vec<f64> {
        vec![self.x, self.y, self.z]
    }
    
    #[wasm_bindgen]
    pub fn velocity(&self) -> Vec<f64> {
        vec![self.vx, self.vy, self.vz]
    }
    
    #[wasm_bindgen]
    pub fn magnitude(&self) -> f64 {
        (self.x * self.x + self.y * self.y + self.z * self.z).sqrt()
    }
    
    #[wasm_bindgen]
    pub fn speed(&self) -> f64 {
        (self.vx * self.vx + self.vy * self.vy + self.vz * self.vz).sqrt()
    }
}

// Time conversion utilities
#[wasm_bindgen]
pub fn calendar_to_et(year: i32, month: i32, day: i32, hour: i32, minute: i32, second: f64) -> f64 {
    let days_since_j2000 = calculate_days_since_j2000(year, month, day);
    let seconds_in_day = hour * 3600 + minute * 60 + second as i32;
    (days_since_j2000 * 86400 + seconds_in_day) as f64
}

#[wasm_bindgen]
pub fn julian_date_to_et(julian_date: f64) -> f64 {
    (julian_date - 2451545.0) * 86400.0
}

#[wasm_bindgen]
pub fn et_to_utc(et: f64, precision: Option<i32>) -> String {
    let _precision = precision.unwrap_or(3);
    format!("2000-01-01T12:00:{:06.3}Z", et / 86400.0)
}

// Kernel management
#[wasm_bindgen]
pub fn load_kernel(data: &[u8], filename: Option<String>) -> Result<(), JsValue> {
    let filename = filename.unwrap_or_else(|| "kernel.bsp".to_string());
    console_log!("Loading kernel: {} ({} bytes)", filename, data.len());
    Ok(())
}

#[wasm_bindgen]
pub fn clear_kernels() {
    console_log!("All kernels cleared");
}

// Coordinate transformations
#[wasm_bindgen]
pub fn rectangular_to_spherical(x: f64, y: f64, z: f64) -> Vec<f64> {
    let r = (x * x + y * y + z * z).sqrt();
    let colat = if r == 0.0 { 0.0 } else { (z / r).acos() };
    let lon = y.atan2(x);
    vec![r, colat, lon]
}

#[wasm_bindgen]
pub fn spherical_to_rectangular(radius: f64, colatitude: f64, longitude: f64) -> Vec<f64> {
    let x = radius * colatitude.sin() * longitude.cos();
    let y = radius * colatitude.sin() * longitude.sin();
    let z = radius * colatitude.cos();
    vec![x, y, z]
}

// Ephemeris calculation functions
#[wasm_bindgen]
pub fn spkezr(
    target: &str,
    et: f64,
    reference_frame: &str,
    aberration_correction: &str,
    observer: &str
) -> Result<StateVector, JsValue> {
    console_log!("SPKEZR: {} relative to {} at ET {}", target, observer, et);
    console_log!("Frame: {}, Correction: {}", reference_frame, aberration_correction);
    
    // Return a dummy state vector for now
    Ok(StateVector::new(
        1000.0 + et * 0.001,
        2000.0 + et * 0.002,
        3000.0 + et * 0.003,
        10.0,
        20.0,
        30.0,
        0.1
    ))
}

#[wasm_bindgen]
pub fn spkpos(
    target: &str,
    et: f64,
    reference_frame: &str,
    aberration_correction: &str,
    observer: &str
) -> Result<JsValue, JsValue> {
    let state = spkezr(target, et, reference_frame, aberration_correction, observer)?;
    
    let result = serde_json::json!({
        "position": state.position(),
        "light_time": state.light_time
    });
    
    Ok(JsValue::from_str(&serde_json::to_string(&result).unwrap()))
}

// Helper functions
fn calculate_days_since_j2000(year: i32, month: i32, day: i32) -> i32 {
    (year - 2000) * 365 + (month - 1) * 30 + day
}

// Test functions
#[wasm_bindgen]
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}

#[wasm_bindgen]
pub fn version() -> String {
    "RustSPICE v0.1.0".to_string()
}
