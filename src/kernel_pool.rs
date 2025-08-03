//! Kernel Pool System for RustSPICE
//! 
//! This module implements the SPICE kernel pool system, which stores and manages
//! kernel variables from text kernels. The kernel pool is a key-value store that
//! holds assignments from text kernels like LSK, FK, IK, SCLK, and meta-kernels.
//! 
//! # CSPICE Equivalents
//! - ldpool_c → load_text_kernel() - Load text kernel file into pool
//! - clpool_c → clear_pool() - Clear all kernel pool variables
//! - pcpool_c → put_character_pool() - Insert character values into pool
//! - pdpool_c → put_double_pool() - Insert double precision values into pool
//! - pipool_c → put_integer_pool() - Insert integer values into pool
//! - gcpool_c → get_character_pool() - Retrieve character values from pool
//! - gdpool_c → get_double_pool() - Retrieve double precision values from pool
//! - gipool_c → get_integer_pool() - Retrieve integer values from pool
//! - dtpool_c → describe_pool_variable() - Get variable info (type, size)
//! - lmpool_c → load_memory_pool() - Load kernel from memory array

use crate::error_handling::{SpiceResult, SpiceError, SpiceErrorType};
use crate::foundation::{SpiceDouble, SpiceInt};
use std::collections::BTreeMap;
use std::vec::Vec;
use std::string::String;
use core::fmt;

/// Maximum number of kernel pool variables
pub const MAX_POOL_VARIABLES: usize = 26003;

/// Maximum length of a kernel pool variable name
pub const MAX_VARIABLE_NAME_LENGTH: usize = 32;

/// Maximum number of values per variable
pub const MAX_VALUES_PER_VARIABLE: usize = 40000;

/// Data types for kernel pool variables
#[derive(Debug, Clone, PartialEq)]
pub enum PoolDataType {
    /// Character string data
    Character,
    /// Double precision numeric data
    Double,
    /// Integer numeric data
    Integer,
}

impl fmt::Display for PoolDataType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let type_str = match self {
            PoolDataType::Character => "C",
            PoolDataType::Double => "N",
            PoolDataType::Integer => "N",
        };
        write!(f, "{}", type_str)
    }
}

/// Values stored in the kernel pool
#[derive(Debug, Clone)]
pub enum PoolValue {
    /// String values
    Characters(Vec<String>),
    /// Double precision values
    Doubles(Vec<SpiceDouble>),
    /// Integer values
    Integers(Vec<SpiceInt>),
}

impl PoolValue {
    /// Get the data type of this pool value
    pub fn data_type(&self) -> PoolDataType {
        match self {
            PoolValue::Characters(_) => PoolDataType::Character,
            PoolValue::Doubles(_) => PoolDataType::Double,
            PoolValue::Integers(_) => PoolDataType::Integer,
        }
    }

    /// Get the number of values
    pub fn len(&self) -> usize {
        match self {
            PoolValue::Characters(values) => values.len(),
            PoolValue::Doubles(values) => values.len(),
            PoolValue::Integers(values) => values.len(),
        }
    }

    /// Check if empty
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

/// The kernel pool - a global store for kernel variables
#[derive(Debug)]
pub struct KernelPool {
    /// Storage for all kernel variables
    variables: BTreeMap<String, PoolValue>,
}

impl Default for KernelPool {
    fn default() -> Self {
        Self::new()
    }
}

impl KernelPool {
    /// Create a new empty kernel pool
    pub fn new() -> Self {
        Self {
            variables: BTreeMap::new(),
        }
    }

    /// Clear all variables from the kernel pool
    pub fn clear(&mut self) {
        self.variables.clear();
    }

    /// Check if a variable exists in the pool
    pub fn contains_variable(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// Get the number of variables in the pool
    pub fn variable_count(&self) -> usize {
        self.variables.len()
    }

    /// Insert character values into the pool
    pub fn put_character(&mut self, name: &str, values: Vec<String>) -> SpiceResult<()> {
        self.validate_variable_name(name)?;
        self.validate_value_count(values.len())?;
        
        self.variables.insert(name.to_uppercase(), PoolValue::Characters(values));
        Ok(())
    }

    /// Insert double precision values into the pool
    pub fn put_double(&mut self, name: &str, values: Vec<SpiceDouble>) -> SpiceResult<()> {
        self.validate_variable_name(name)?;
        self.validate_value_count(values.len())?;
        
        self.variables.insert(name.to_uppercase(), PoolValue::Doubles(values));
        Ok(())
    }

    /// Insert integer values into the pool
    pub fn put_integer(&mut self, name: &str, values: Vec<SpiceInt>) -> SpiceResult<()> {
        self.validate_variable_name(name)?;
        self.validate_value_count(values.len())?;
        
        self.variables.insert(name.to_uppercase(), PoolValue::Integers(values));
        Ok(())
    }

    /// Get character values from the pool
    pub fn get_character(&self, name: &str, start: usize, max_values: usize) -> SpiceResult<(Vec<String>, bool)> {
        let upper_name = name.to_uppercase();
        
        if let Some(value) = self.variables.get(&upper_name) {
            match value {
                PoolValue::Characters(chars) => {
                    let end = std::cmp::min(start + max_values, chars.len());
                    if start >= chars.len() {
                        Ok((Vec::new(), true))
                    } else {
                        Ok((chars[start..end].to_vec(), true))
                    }
                },
                _ => Err(SpiceError::new(
                    SpiceErrorType::InvalidDataType,
                    format!("Variable '{}' is not of character type", name)
                ))
            }
        } else {
            Ok((Vec::new(), false))
        }
    }

    /// Get double precision values from the pool
    pub fn get_double(&self, name: &str, start: usize, max_values: usize) -> SpiceResult<(Vec<SpiceDouble>, bool)> {
        let upper_name = name.to_uppercase();
        
        if let Some(value) = self.variables.get(&upper_name) {
            match value {
                PoolValue::Doubles(doubles) => {
                    let end = std::cmp::min(start + max_values, doubles.len());
                    if start >= doubles.len() {
                        Ok((Vec::new(), true))
                    } else {
                        Ok((doubles[start..end].to_vec(), true))
                    }
                },
                PoolValue::Integers(ints) => {
                    // Allow retrieving integers as doubles
                    let end = std::cmp::min(start + max_values, ints.len());
                    if start >= ints.len() {
                        Ok((Vec::new(), true))
                    } else {
                        let doubles: Vec<SpiceDouble> = ints[start..end].iter().map(|&i| i as SpiceDouble).collect();
                        Ok((doubles, true))
                    }
                },
                _ => Err(SpiceError::new(
                    SpiceErrorType::InvalidDataType,
                    format!("Variable '{}' is not of numeric type", name)
                ))
            }
        } else {
            Ok((Vec::new(), false))
        }
    }

    /// Get integer values from the pool
    pub fn get_integer(&self, name: &str, start: usize, max_values: usize) -> SpiceResult<(Vec<SpiceInt>, bool)> {
        let upper_name = name.to_uppercase();
        
        if let Some(value) = self.variables.get(&upper_name) {
            match value {
                PoolValue::Integers(ints) => {
                    let end = std::cmp::min(start + max_values, ints.len());
                    if start >= ints.len() {
                        Ok((Vec::new(), true))
                    } else {
                        Ok((ints[start..end].to_vec(), true))
                    }
                },
                PoolValue::Doubles(doubles) => {
                    // Allow retrieving doubles as integers (with conversion)
                    let end = std::cmp::min(start + max_values, doubles.len());
                    if start >= doubles.len() {
                        Ok((Vec::new(), true))
                    } else {
                        let ints: Vec<SpiceInt> = doubles[start..end].iter().map(|&d| d as SpiceInt).collect();
                        Ok((ints, true))
                    }
                },
                _ => Err(SpiceError::new(
                    SpiceErrorType::InvalidDataType,
                    format!("Variable '{}' is not of numeric type", name)
                ))
            }
        } else {
            Ok((Vec::new(), false))
        }
    }

    /// Get information about a pool variable (type and size)
    pub fn describe_variable(&self, name: &str) -> SpiceResult<(bool, usize, PoolDataType)> {
        let upper_name = name.to_uppercase();
        
        if let Some(value) = self.variables.get(&upper_name) {
            Ok((true, value.len(), value.data_type()))
        } else {
            Ok((false, 0, PoolDataType::Character))
        }
    }

    /// Remove a variable from the pool
    pub fn delete_variable(&mut self, name: &str) -> SpiceResult<bool> {
        let upper_name = name.to_uppercase();
        Ok(self.variables.remove(&upper_name).is_some())
    }

    /// Get all variable names matching a pattern
    pub fn get_variable_names(&self, pattern: &str) -> SpiceResult<Vec<String>> {
        let mut names = Vec::new();
        
        // Simple pattern matching - for now just handle "*" wildcard
        if pattern == "*" {
            names.extend(self.variables.keys().cloned());
        } else if pattern.contains('*') {
            // Basic wildcard support
            let prefix = pattern.replace('*', "");
            for name in self.variables.keys() {
                if name.starts_with(&prefix.to_uppercase()) {
                    names.push(name.clone());
                }
            }
        } else {
            // Exact match
            let upper_pattern = pattern.to_uppercase();
            if self.variables.contains_key(&upper_pattern) {
                names.push(upper_pattern);
            }
        }
        
        names.sort();
        Ok(names)
    }

    /// Load text kernel content from memory
    pub fn load_from_memory(&mut self, lines: &[String]) -> SpiceResult<()> {
        let mut in_data_section = false;
        let mut current_assignment = String::new();
        
        for line in lines {
            let trimmed = line.trim();
            
            // Skip comments and empty lines
            if trimmed.is_empty() || trimmed.starts_with('#') {
                continue;
            }
            
            // Check for section markers
            if trimmed.starts_with("\\begindata") {
                in_data_section = true;
                continue;
            } else if trimmed.starts_with("\\begintext") {
                in_data_section = false;
                continue;
            }
            
            if !in_data_section {
                continue;
            }
            
            // Handle line continuation
            if trimmed.ends_with('+') {
                current_assignment.push_str(&trimmed[..trimmed.len()-1]);
                continue;
            } else {
                current_assignment.push_str(trimmed);
            }
            
            // Process complete assignment
            if !current_assignment.is_empty() {
                if let Err(e) = self.parse_assignment(&current_assignment) {
                    // Log warning and continue processing other assignments
                    eprintln!("Warning: Skipping problematic assignment '{}': {}", 
                             current_assignment.split('=').next().unwrap_or("").trim(), e);
                }
                current_assignment.clear();
            }
        }
        
        Ok(())
    }

    /// Parse a single assignment statement
    fn parse_assignment(&mut self, assignment: &str) -> SpiceResult<()> {
        if let Some(eq_pos) = assignment.find('=') {
            let var_name = assignment[..eq_pos].trim().to_uppercase();
            let value_part = assignment[eq_pos + 1..].trim();
            
            self.validate_variable_name(&var_name)?;
            
            // Parse the value(s)
            if value_part.starts_with('(') && value_part.ends_with(')') {
                // Array assignment
                let inner = &value_part[1..value_part.len()-1];
                self.parse_array_values(&var_name, inner)?;
            } else {
                // Single value assignment
                self.parse_single_value(&var_name, value_part)?;
            }
        }
        
        Ok(())
    }

    /// Parse array values from parentheses
    fn parse_array_values(&mut self, var_name: &str, values_str: &str) -> SpiceResult<()> {
        let mut values = Vec::new();
        let mut current_value = String::new();
        let mut in_quotes = false;
        let mut quote_char = '"';
        
        for ch in values_str.chars() {
            match ch {
                '"' | '\'' if !in_quotes => {
                    in_quotes = true;
                    quote_char = ch;
                },
                '"' | '\'' if in_quotes && ch == quote_char => {
                    in_quotes = false;
                },
                ',' if !in_quotes => {
                    if !current_value.trim().is_empty() {
                        values.push(current_value.trim().to_string());
                    }
                    current_value.clear();
                },
                _ => {
                    current_value.push(ch);
                }
            }
        }
        
        // Add final value
        if !current_value.trim().is_empty() {
            values.push(current_value.trim().to_string());
        }
        
        // Determine type and store
        if !values.is_empty() {
            // Check if any value contains quotes, indicating string values
            let has_quotes = values.iter().any(|v| {
                v.contains('"') || v.contains('\'')
            });
            
            if has_quotes {
                // Character values
                let char_values: Vec<String> = values.into_iter()
                    .map(|v| v.trim_matches(|c| c == '"' || c == '\'').to_string())
                    .collect();
                self.put_character(var_name, char_values)?;
            } else {
                // Try to parse as numeric values
                let double_values: Result<Vec<SpiceDouble>, _> = values.iter()
                    .map(|v| self.parse_spice_number(v))
                    .collect();
                
                match double_values {
                    Ok(doubles) => self.put_double(var_name, doubles)?,
                    Err(_) => {
                        // If numeric parsing fails, treat as character values
                        self.put_character(var_name, values)?;
                    }
                }
            }
        }
        
        Ok(())
    }

    /// Parse a single value
    fn parse_single_value(&mut self, var_name: &str, value_str: &str) -> SpiceResult<()> {
        let trimmed = value_str.trim();
        
        if trimmed.starts_with('"') || trimmed.starts_with('\'') {
            // Character value
            let char_value = trimmed.trim_matches(|c| c == '"' || c == '\'').to_string();
            self.put_character(var_name, vec![char_value])?;
        } else {
            // Numeric value - handle SPICE/FORTRAN D notation
            match self.parse_spice_number(trimmed) {
                Ok(double_val) => self.put_double(var_name, vec![double_val])?,
                Err(_) => {
                    return Err(SpiceError::new(
                        SpiceErrorType::InvalidFormat,
                        format!("Could not parse value '{}' for variable '{}'", trimmed, var_name)
                    ));
                }
            }
        }
        
        Ok(())
    }

    /// Parse SPICE/FORTRAN number format (handles 'D' for scientific notation)
    fn parse_spice_number(&self, value_str: &str) -> Result<SpiceDouble, std::num::ParseFloatError> {
        // Convert FORTRAN 'D' scientific notation to Rust 'E' notation
        let rust_format = value_str.replace('D', "E").replace('d', "e");
        rust_format.parse::<SpiceDouble>()
    }

    /// Validate variable name length and characters
    fn validate_variable_name(&self, name: &str) -> SpiceResult<()> {
        if name.is_empty() {
            return Err(SpiceError::new(
                SpiceErrorType::InvalidArgument,
                "Variable name cannot be empty".to_string()
            ));
        }
        
        if name.len() > MAX_VARIABLE_NAME_LENGTH {
            return Err(SpiceError::new(
                SpiceErrorType::InvalidArgument,
                format!("Variable name '{}' exceeds maximum length of {} characters", name, MAX_VARIABLE_NAME_LENGTH)
            ));
        }
        
        Ok(())
    }

    /// Validate number of values
    fn validate_value_count(&self, count: usize) -> SpiceResult<()> {
        if count > MAX_VALUES_PER_VARIABLE {
            return Err(SpiceError::new(
                SpiceErrorType::InvalidArgument,
                format!("Number of values ({}) exceeds maximum of {}", count, MAX_VALUES_PER_VARIABLE)
            ));
        }
        
        Ok(())
    }
}

/// Global kernel pool instance
static mut GLOBAL_POOL: Option<KernelPool> = None;
static mut POOL_INITIALIZED: bool = false;

/// Initialize the global kernel pool
pub fn initialize_pool() -> SpiceResult<()> {
    unsafe {
        GLOBAL_POOL = Some(KernelPool::new());
        POOL_INITIALIZED = true;
    }
    Ok(())
}

/// Check if the kernel pool is initialized
pub fn is_pool_initialized() -> bool {
    unsafe { POOL_INITIALIZED }
}

/// Get mutable access to the global pool
unsafe fn get_global_pool_mut() -> SpiceResult<&'static mut KernelPool> {
    if !POOL_INITIALIZED {
        return Err(SpiceError::new(
            SpiceErrorType::PoolNotInitialized,
            "Kernel pool has not been initialized. Call initialize_pool() first.".to_string()
        ));
    }
    
    GLOBAL_POOL.as_mut().ok_or_else(|| SpiceError::new(
        SpiceErrorType::PoolNotInitialized,
        "Global kernel pool is None".to_string()
    ))
}

/// Get immutable access to the global pool
unsafe fn get_global_pool() -> SpiceResult<&'static KernelPool> {
    if !POOL_INITIALIZED {
        return Err(SpiceError::new(
            SpiceErrorType::PoolNotInitialized,
            "Kernel pool has not been initialized. Call initialize_pool() first.".to_string()
        ));
    }
    
    GLOBAL_POOL.as_ref().ok_or_else(|| SpiceError::new(
        SpiceErrorType::PoolNotInitialized,
        "Global kernel pool is None".to_string()
    ))
}

// Public API functions that mirror CSPICE

/// Clear the kernel pool (equivalent to clpool_c)
pub fn clear_pool() -> SpiceResult<()> {
    unsafe {
        let pool = get_global_pool_mut()?;
        pool.clear();
    }
    Ok(())
}

/// Put character values into the kernel pool (equivalent to pcpool_c)
pub fn put_character_pool(name: &str, values: Vec<String>) -> SpiceResult<()> {
    unsafe {
        let pool = get_global_pool_mut()?;
        pool.put_character(name, values)
    }
}

/// Put double precision values into the kernel pool (equivalent to pdpool_c)
pub fn put_double_pool(name: &str, values: Vec<SpiceDouble>) -> SpiceResult<()> {
    unsafe {
        let pool = get_global_pool_mut()?;
        pool.put_double(name, values)
    }
}

/// Put integer values into the kernel pool (equivalent to pipool_c)
pub fn put_integer_pool(name: &str, values: Vec<SpiceInt>) -> SpiceResult<()> {
    unsafe {
        let pool = get_global_pool_mut()?;
        pool.put_integer(name, values)
    }
}

/// Get character values from the kernel pool (equivalent to gcpool_c)
pub fn get_character_pool(name: &str, start: usize, max_values: usize) -> SpiceResult<(Vec<String>, bool)> {
    unsafe {
        let pool = get_global_pool()?;
        pool.get_character(name, start, max_values)
    }
}

/// Get double precision values from the kernel pool (equivalent to gdpool_c)
pub fn get_double_pool(name: &str, start: usize, max_values: usize) -> SpiceResult<(Vec<SpiceDouble>, bool)> {
    unsafe {
        let pool = get_global_pool()?;
        pool.get_double(name, start, max_values)
    }
}

/// Get integer values from the kernel pool (equivalent to gipool_c)
pub fn get_integer_pool(name: &str, start: usize, max_values: usize) -> SpiceResult<(Vec<SpiceInt>, bool)> {
    unsafe {
        let pool = get_global_pool()?;
        pool.get_integer(name, start, max_values)
    }
}

/// Get information about a pool variable (equivalent to dtpool_c)
pub fn describe_pool_variable(name: &str) -> SpiceResult<(bool, usize, PoolDataType)> {
    unsafe {
        let pool = get_global_pool()?;
        pool.describe_variable(name)
    }
}

/// Check if a variable exists in the pool (equivalent to expool_c)
pub fn exists_in_pool(name: &str) -> SpiceResult<bool> {
    unsafe {
        let pool = get_global_pool()?;
        Ok(pool.contains_variable(name))
    }
}

/// Delete a variable from the pool (equivalent to dvpool_c)
pub fn delete_pool_variable(name: &str) -> SpiceResult<bool> {
    unsafe {
        let pool = get_global_pool_mut()?;
        pool.delete_variable(name)
    }
}

/// Get variable names matching a pattern (equivalent to gnpool_c)
pub fn get_pool_variable_names(pattern: &str) -> SpiceResult<Vec<String>> {
    unsafe {
        let pool = get_global_pool()?;
        pool.get_variable_names(pattern)
    }
}

/// Load text kernel from memory (equivalent to lmpool_c)
pub fn load_memory_pool(lines: &[String]) -> SpiceResult<()> {
    unsafe {
        let pool = get_global_pool_mut()?;
        pool.load_from_memory(lines)
    }
}

/// Load text kernel from file (equivalent to ldpool_c)
pub fn load_text_kernel(content: &str) -> SpiceResult<()> {
    let lines: Vec<String> = content.lines().map(|line| line.to_string()).collect();
    load_memory_pool(&lines)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_initialize_pool() {
        let result = initialize_pool();
        assert!(result.is_ok());
        assert!(is_pool_initialized());
    }

    #[test]
    fn test_character_pool_operations() {
        initialize_pool().unwrap();
        clear_pool().unwrap();

        // Put character values
        let values = vec!["HELLO".to_string(), "WORLD".to_string()];
        put_character_pool("TEST_CHAR", values.clone()).unwrap();

        // Get character values
        let (retrieved, found) = get_character_pool("TEST_CHAR", 0, 10).unwrap();
        assert!(found);
        assert_eq!(retrieved, values);

        // Check variable exists
        assert!(exists_in_pool("TEST_CHAR").unwrap());

        // Get variable info
        let (found, size, data_type) = describe_pool_variable("TEST_CHAR").unwrap();
        assert!(found);
        assert_eq!(size, 2);
        assert_eq!(data_type, PoolDataType::Character);
    }

    #[test]
    fn test_double_pool_operations() {
        initialize_pool().unwrap();
        clear_pool().unwrap();

        // Put double values
        let values = vec![1.0, 2.5, 3.14159];
        put_double_pool("TEST_DOUBLE", values.clone()).unwrap();

        // Get double values
        let (retrieved, found) = get_double_pool("TEST_DOUBLE", 0, 10).unwrap();
        assert!(found);
        assert_eq!(retrieved, values);

        // Get partial values
        let (partial, found) = get_double_pool("TEST_DOUBLE", 1, 1).unwrap();
        assert!(found);
        assert_eq!(partial, vec![2.5]);
    }

    #[test]
    fn test_integer_pool_operations() {
        initialize_pool().unwrap();
        clear_pool().unwrap();

        // Put integer values
        let values = vec![1, 2, 3, 4, 5];
        put_integer_pool("TEST_INT", values.clone()).unwrap();

        // Get integer values
        let (retrieved, found) = get_integer_pool("TEST_INT", 0, 10).unwrap();
        assert!(found);
        assert_eq!(retrieved, values);
    }

    #[test]
    fn test_text_kernel_parsing() {
        initialize_pool().unwrap();
        clear_pool().unwrap();

        let kernel_content = r#"
\begindata

TEST_STRING = 'Hello World'
TEST_ARRAY = ( 1.0, 2.0, 3.0 )
TEST_CHARS = ( 'A', 'B', 'C' )

\begintext

This is a comment section.
"#;

        load_text_kernel(kernel_content).unwrap();

        // Check string value
        let (string_val, found) = get_character_pool("TEST_STRING", 0, 1).unwrap();
        assert!(found);
        assert_eq!(string_val, vec!["Hello World"]);

        // Check numeric array
        let (double_val, found) = get_double_pool("TEST_ARRAY", 0, 10).unwrap();
        assert!(found);
        assert_eq!(double_val, vec![1.0, 2.0, 3.0]);

        // Check character array
        let (char_val, found) = get_character_pool("TEST_CHARS", 0, 10).unwrap();
        assert!(found);
        assert_eq!(char_val, vec!["A", "B", "C"]);
    }

    #[test]
    fn test_variable_name_patterns() {
        initialize_pool().unwrap();
        clear_pool().unwrap();

        // Add some test variables
        put_double_pool("BODY399_GM", vec![398600.4415]).unwrap();
        put_double_pool("BODY301_GM", vec![4902.8]).unwrap();
        put_character_pool("FRAME_NAME", vec!["J2000".to_string()]).unwrap();

        // Test wildcard pattern
        let names = get_pool_variable_names("BODY*").unwrap();
        assert!(names.contains(&"BODY399_GM".to_string()));
        assert!(names.contains(&"BODY301_GM".to_string()));
        assert!(!names.contains(&"FRAME_NAME".to_string()));

        // Test all variables
        let all_names = get_pool_variable_names("*").unwrap();
        assert_eq!(all_names.len(), 3);
    }

    #[test]
    fn test_delete_variable() {
        initialize_pool().unwrap();
        clear_pool().unwrap();

        put_double_pool("TO_DELETE", vec![1.0]).unwrap();
        assert!(exists_in_pool("TO_DELETE").unwrap());

        let deleted = delete_pool_variable("TO_DELETE").unwrap();
        assert!(deleted);
        assert!(!exists_in_pool("TO_DELETE").unwrap());

        // Try to delete non-existent variable
        let deleted = delete_pool_variable("NONEXISTENT").unwrap();
        assert!(!deleted);
    }

    #[test]
    fn test_clear_pool() {
        initialize_pool().unwrap();
        clear_pool().unwrap();

        // Add some variables
        put_double_pool("VAR1", vec![1.0]).unwrap();
        put_character_pool("VAR2", vec!["test".to_string()]).unwrap();

        let all_names = get_pool_variable_names("*").unwrap();
        assert_eq!(all_names.len(), 2);

        // Clear the pool
        clear_pool().unwrap();

        let all_names = get_pool_variable_names("*").unwrap();
        assert_eq!(all_names.len(), 0);
    }
}
