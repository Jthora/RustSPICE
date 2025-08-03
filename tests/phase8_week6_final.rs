// Phase 8 Week 6: Final Integration Tests for Advanced Mathematical Functions
// Test that Phase 8 mathematical functions integrate properly with foundation modules

use rust_spice::advanced_math::*;
use rust_spice::foundation::*;
use rust_spice::*;

/// Test mathematical optimization integration with foundation vectors
#[test]
fn test_optimization_integration() {
    // Test simple quadratic function f(x) = (x-2)² with minimum at x=2
    let func = |x: f64| (x - 2.0).powi(2);
    let dfunc = |x: f64| 2.0 * (x - 2.0);

    // Newton-Raphson for root finding
    let root = OptimizationMethods::newton_raphson(&func, &dfunc, 1.0, 1e-10, 100).unwrap();
    assert!((root - 2.0).abs() < 1e-9);

    // Golden section search for minimum
    let minimum = OptimizationMethods::golden_section_search(&func, 0.0, 4.0, 1e-8, 1000).unwrap();
    assert!((minimum - 2.0).abs() < 1e-7);
}

/// Test numerical differentiation with foundation data structures
#[test]
fn test_differentiation_integration() {
    // Test function f(x) = x³ + 2x² + x + 1
    let func = |x: f64| x.powi(3) + 2.0 * x.powi(2) + x + 1.0;

    let x = 2.0;
    let forward_diff = NumericalDifferentiation::forward_difference(&func, x, 1e-6);
    let backward_diff = NumericalDifferentiation::backward_difference(&func, x, 1e-6);
    let central_diff = NumericalDifferentiation::central_difference(&func, x, 1e-6);

    // Analytical derivative at x=2: f'(x) = 3x² + 4x + 1 = 12 + 8 + 1 = 21
    let analytical = 21.0;
    
    assert!((forward_diff - analytical).abs() < 1e-3);
    assert!((backward_diff - analytical).abs() < 1e-3);
    assert!((central_diff - analytical).abs() < 1e-5);
}

/// Test curve fitting with foundation data structures  
#[test]
fn test_curve_fitting_integration() {
    // Test linear least squares with foundation vectors
    let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
    let y = vec![5.0, 7.0, 9.0, 11.0, 13.0]; // y = 2x + 3

    let coeffs = OptimizationMethods::linear_least_squares(&x, &y).unwrap();
    assert!((coeffs.0 - 3.0).abs() < 1e-10); // intercept
    assert!((coeffs.1 - 2.0).abs() < 1e-10); // slope
}

/// Test Chebyshev polynomials with foundation matrix operations
#[test]
fn test_chebyshev_with_matrices() {
    let chebyshev = ChebyshevPolynomials::new(5);
    let coefficients = vec![1.0, 0.5, 0.25, 0.125, 0.0625];

    // Test evaluation
    let result = chebyshev.evaluate_series(&coefficients, 0.5).unwrap();
    assert!(result.is_finite() && result > 0.0);

    // Test with matrix-like operations using foundation vectors
    let identity = SpiceMatrix3x3::identity();
    let test_vec = SpiceVector3::new(1.0, 0.5, 0.25);
    let transformed = identity.multiply_vector(&test_vec);
    
    // Should be unchanged by identity transformation
    assert!((transformed.x() - test_vec.x()).abs() < 1e-15);
    assert!((transformed.y() - test_vec.y()).abs() < 1e-15);
    assert!((transformed.z() - test_vec.z()).abs() < 1e-15);
}

/// Test interpolation methods with foundation data
#[test]
fn test_interpolation_methods() {
    // Test Hermite interpolation
    let mut hermite = HermiteInterpolator::new();
    hermite.add_point(0.0, 0.0, 0.0);
    hermite.add_point(1.0, 1.0, 2.0);
    hermite.add_point(2.0, 4.0, 4.0);
    
    let result = hermite.evaluate(1.5).unwrap();
    assert!(result.is_finite());

    // Test Lagrange interpolation
    let mut lagrange = LagrangeInterpolator::new();
    lagrange.add_point(0.0, 0.0);
    lagrange.add_point(1.0, 1.0);
    lagrange.add_point(2.0, 4.0);
    
    let result = lagrange.evaluate(1.5).unwrap();
    assert!(result.is_finite());
}

/// Test performance integration
#[test]
fn test_performance_integration() {
    use std::time::Instant;

    let start = Instant::now();

    // Run multiple mathematical operations in sequence
    for _ in 0..50 {
        // Vector operations
        let vec1 = SpiceVector3::new(1.0, 2.0, 3.0);
        let vec2 = SpiceVector3::new(2.0, 3.0, 4.0);
        let _cross = vec1.cross(&vec2);

        // Matrix operations
        let matrix = SpiceMatrix3x3::identity();
        let _result = matrix.multiply_vector(&vec1);

        // Simple optimization
        let simple_func = |x: f64| (x - 1.0).powi(2);
        let _min = OptimizationMethods::golden_section_search(&simple_func, 0.0, 2.0, 1e-6, 100).unwrap();

        // Numerical differentiation
        let _deriv = NumericalDifferentiation::central_difference(&simple_func, 1.0, 1e-8);
    }

    let duration = start.elapsed();

    // Should complete 50 iterations quickly (< 50ms)
    assert!(duration.as_millis() < 50, "Performance regression: {:?}", duration);
}

/// Test mathematical consistency across all modules
#[test]
fn test_mathematical_consistency() {
    // Test that same mathematical operations give consistent results
    let test_func = |x: f64| x.powi(3) - 3.0 * x.powi(2) + 2.0 * x; // x(x-1)(x-2), roots at 0,1,2
    let dfunc = |x: f64| 3.0 * x.powi(2) - 6.0 * x + 2.0;

    // Find roots using different methods - should all give same result
    let root1_newton = OptimizationMethods::newton_raphson(&test_func, &dfunc, 0.1, 1e-12, 100).unwrap();
    let root1_brent = OptimizationMethods::brent_method(&test_func, -0.5, 0.5, 1e-12, 100).unwrap();
    
    // Both methods should find the root at x=0
    assert!((root1_newton - 0.0).abs() < 1e-10);
    assert!((root1_brent - 0.0).abs() < 1e-10);
    assert!((root1_newton - root1_brent).abs() < 1e-10);
}

/// Test cross-module compatibility
#[test]
fn test_cross_module_compatibility() {
    // Test that Phase 8 functions work with Phase 1-7 foundation types
    
    // Vector creation and manipulation
    let mut points = Vec::new();
    for i in 0..5 {
        let vec = SpiceVector3::new(i as f64, (i*i) as f64, (i*i*i) as f64);
        points.push((vec.x(), vec.y()));
    }
    
    // Use optimization with vector-derived data
    let coeffs = OptimizationMethods::linear_least_squares(
        &points.iter().map(|(x, _)| *x).collect::<Vec<_>>(),
        &points.iter().map(|(_, y)| *y).collect::<Vec<_>>()
    ).unwrap();
    
    // Should produce reasonable coefficients
    assert!(coeffs.0.is_finite());
    assert!(coeffs.1.is_finite());
    
    // Test with matrix operations
    let matrix = SpiceMatrix3x3::from_rows([
        [1.0, 0.0, 0.0],
        [0.0, 1.0, 0.0],
        [0.0, 0.0, 1.0]
    ]);
    
    for i in 0..3 {
        let test_vec = SpiceVector3::new(i as f64, (i+1) as f64, (i+2) as f64);
        let result = matrix.multiply_vector(&test_vec);
        
        // Identity matrix should preserve vector
        assert!((result.x() - test_vec.x()).abs() < 1e-15);
        assert!((result.y() - test_vec.y()).abs() < 1e-15);
        assert!((result.z() - test_vec.z()).abs() < 1e-15);
    }
}
