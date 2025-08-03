/// Phase 8 Week 6 Final Integration Tests
/// 
/// Simplified integration tests to validate that Phase 8 advanced mathematical 
/// functions work correctly with existing RustSPICE modules

use rust_spice::advanced_math::*;
use rust_spice::foundation::*;

#[cfg(test)]
mod phase8_week6_integration {
    use super::*;

    /// Test that all Week 5 optimization algorithms are working
    #[test]
    fn test_week5_algorithms_functional() {
        // Test Newton-Raphson
        let func = |x: f64| x.powi(2) - 4.0;
        let dfunc = |x: f64| 2.0 * x;
        let root = OptimizationMethods::newton_raphson(&func, &dfunc, 1.0, 1e-10, 100).unwrap();
        assert!((root - 2.0).abs() < 1e-10);

        // Test Secant method
        let root2 = OptimizationMethods::secant_method(&func, 1.0, 3.0, 1e-10, 100).unwrap();
        assert!((root2 - 2.0).abs() < 1e-8);

        // Test Brent's method
        let root3 = OptimizationMethods::brent_method(&func, 0.0, 3.0, 1e-10, 100).unwrap();
        assert!((root3 - 2.0).abs() < 1e-10);

        // Test Golden section search
        let min_func = |x: f64| (x - 2.0).powi(2) + 1.0;
        let minimum = OptimizationMethods::golden_section_search(&min_func, 0.0, 4.0, 1e-8, 1000).unwrap();
        assert!((minimum - 2.0).abs() < 1e-6);

        // Test Nelder-Mead
        let quad_func = |x: &[f64]| (x[0] - 1.0).powi(2) + (x[1] - 1.0).powi(2);
        let simplex = vec![vec![0.0, 0.0], vec![1.5, 0.0], vec![0.0, 1.5]];
        let result = OptimizationMethods::nelder_mead(quad_func, &simplex, 1e-8, 1000).unwrap();
        assert!((result[0] - 1.0).abs() < 1e-2);
        assert!((result[1] - 1.0).abs() < 1e-2);
    }

    /// Test that numerical differentiation works with foundation vector operations
    #[test]
    fn test_numerical_differentiation_with_vectors() {
        // Test function that uses vector operations
        let vector_magnitude_func = |x: f64| {
            let vec = SpiceVector3::new(x, 2.0, 3.0);
            vec.magnitude()
        };

        // Test forward difference
        let forward_diff = NumericalDifferentiation::forward_difference(&vector_magnitude_func, 1.0, 1e-8);
        
        // Test backward difference
        let backward_diff = NumericalDifferentiation::backward_difference(&vector_magnitude_func, 1.0, 1e-8);
        
        // Test central difference
        let central_diff = NumericalDifferentiation::central_difference(&vector_magnitude_func, 1.0, 1e-8);

        // All should give approximately the same result
        assert!((forward_diff - backward_diff).abs() < 1e-6);
        assert!((central_diff - backward_diff).abs() < 1e-6);

        // Analytical derivative at x=1: d/dx sqrt(x² + 4 + 9) = x/sqrt(x² + 13)
        let analytical = 1.0f64 / (1.0f64 + 4.0f64 + 9.0f64).sqrt();
        assert!((central_diff - analytical).abs() < 1e-6);
    }

    /// Test curve fitting with foundation data structures
    #[test]
    fn test_curve_fitting_integration() {
        let optimization = OptimizationMethods::default();
        
        // Test linear least squares with foundation vectors
        let x = vec![1.0, 2.0, 3.0, 4.0, 5.0];
        let y = vec![5.0, 7.0, 9.0, 11.0, 13.0]; // y = 2x + 3
        
        let coeffs = optimization.linear_least_squares(&x, &y).unwrap();
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

        // Test with matrix-like operations (coefficient transformation)
        let identity = SpiceMatrix3x3::identity();
        let test_vec = SpiceVector3::new(1.0, 0.5, 0.25);
        let transformed = identity.multiply_vector(&test_vec);
        
        // Should be unchanged by identity transformation
        assert!((transformed.x() - test_vec.x()).abs() < 1e-15);
        assert!((transformed.y() - test_vec.y()).abs() < 1e-15);
        assert!((transformed.z() - test_vec.z()).abs() < 1e-15);
    }

    /// Test interpolation methods
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

    /// Test that gradient computation works with multivariate functions
    #[test]
    fn test_gradient_computation() {
        // Test function f(x,y) = x² + 2xy + y²
        let func = |vars: &[f64]| -> f64 {
            if vars.len() != 2 { return 0.0; }
            let x = vars[0];
            let y = vars[1];
            x*x + 2.0*x*y + y*y
        };

        let point = [1.0, 2.0];
        let gradient = NumericalDifferentiation::gradient(&func, &point, 1e-8);
        
        // Analytical gradient: [2x + 2y, 2x + 2y] = [6, 6] at (1,2)
        assert!((gradient[0] - 6.0).abs() < 1e-6);
        assert!((gradient[1] - 6.0).abs() < 1e-6);
    }

    /// Test Hessian computation
    #[test]
    fn test_hessian_computation() {
        // Test function f(x,y) = x² + xy + y²
        let func = |vars: &[f64]| -> f64 {
            if vars.len() != 2 { return 0.0; }
            let x = vars[0];
            let y = vars[1];
            x*x + x*y + y*y
        };

        let point = [1.0, 1.0];
        let hessian = NumericalDifferentiation::hessian(&func, &point, 1e-4);

        // Analytical Hessian: [[2, 1], [1, 2]]
        assert!((hessian[0][0] - 2.0).abs() < 1e-4);
        assert!((hessian[0][1] - 1.0).abs() < 1e-4);
        assert!((hessian[1][0] - 1.0).abs() < 1e-4);
        assert!((hessian[1][1] - 2.0).abs() < 1e-4);
    }

    /// Performance integration test
    #[test]
    fn test_performance_integration() {
        use std::time::Instant;

        let start = Instant::now();

        // Run multiple mathematical operations in sequence
        for _ in 0..100 {
            // Vector operations
            let vec1 = SpiceVector3::new(1.0, 2.0, 3.0);
            let vec2 = SpiceVector3::new(2.0, 3.0, 4.0);
            let _cross = vec1.cross(&vec2);

            // Matrix operations
            let matrix = SpiceMatrix3x3::identity();
            let _result = matrix.multiply_vector(&vec1);

            // Optimization
            let simple_func = |x: f64| (x - 1.0).powi(2);
            let _min = OptimizationMethods::golden_section_search(&simple_func, 0.0, 2.0, 1e-6, 100).unwrap();

            // Numerical differentiation
            let _deriv = NumericalDifferentiation::central_difference(&simple_func, 1.0, 1e-8);
        }

        let duration = start.elapsed();

        // Should complete 100 iterations quickly (< 100ms)
        assert!(duration.as_millis() < 100, "Performance regression: {:?}", duration);
    }

    /// Test mathematical consistency across all modules
    #[test]
    fn test_mathematical_consistency() {
        // Test that same mathematical operations give consistent results
        let test_func = |x: f64| x.powi(3) - 3.0 * x.powi(2) + 2.0 * x; // x(x-1)(x-2), roots at 0,1,2

        // Find roots using different methods
        let dfunc = |x: f64| 3.0 * x.powi(2) - 6.0 * x + 2.0;

        let root1_newton = OptimizationMethods::newton_raphson(&test_func, &dfunc, 0.1, 1e-12, 100).unwrap();
        let root1_brent = OptimizationMethods::brent_method(&test_func, -0.5, 0.5, 1e-12, 100).unwrap();
        let root1_secant = OptimizationMethods::secant_method(&test_func, -0.1, 0.1, 1e-12, 100).unwrap();

        // All methods should find the same root (x=0)
        assert!((root1_newton - 0.0).abs() < 1e-10);
        assert!((root1_brent - 0.0).abs() < 1e-10);
        assert!((root1_secant - 0.0).abs() < 1e-10);

        // Methods should agree with each other
        assert!((root1_newton - root1_brent).abs() < 1e-10);
        assert!((root1_newton - root1_secant).abs() < 1e-10);
    }
}
