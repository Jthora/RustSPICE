/// Phase 8 Integration Tests: Advanced Mathematical Functions
/// 
/// This test suite validates that the advanced mathematical functions from Phase 8
/// integrate properly with existing RustSPICE modules and maintain CSPICE compatibility.

use rust_spice::advanced_math::*;
use rust_spice::foundation::*;
use rust_spice::time_system::*;
use rust_spice::coordinates::*;

#[cfg(test)]
mod phase8_integration_tests {
    use super::*;

    /// Test that Chebyshev polynomials integrate with time calculations
    #[test]
    fn test_chebyshev_with_time_system() {
        // Create a Chebyshev polynomial for time interpolation
        let coefficients = vec![1.0, 0.5, 0.25, 0.125];
        let domain = (-1.0, 1.0);
        
        let chebyshev = ChebyshevPolynomials::new(coefficients, domain);
        
        // Test evaluation at different time-normalized points
        let result_start = chebyshev.evaluate(-1.0).unwrap();
        let result_mid = chebyshev.evaluate(0.0).unwrap();
        let result_end = chebyshev.evaluate(1.0).unwrap();
        
        // Verify reasonable interpolation behavior
        assert!((result_start - 0.375).abs() < 1e-12); // Sum of alternating coefficients
        assert!((result_mid - 1.0).abs() < 1e-12);     // First coefficient only
        assert!((result_end - 1.875).abs() < 1e-12);   // Sum of all coefficients
        
        // Test integration with time derivatives
        let derivative = chebyshev.derivative(0.0).unwrap();
        assert!((derivative - 0.5).abs() < 1e-12); // Should be second coefficient
    }

    /// Test that matrix operations integrate with optimization methods
    #[test]
    fn test_matrix_optimization_integration() {
        // Define a quadratic function using matrix operations
        let a = SpiceMatrix3x3::from_array([
            [2.0, 1.0, 0.0],
            [1.0, 2.0, 0.0],
            [0.0, 0.0, 1.0]
        ]);
        
        let objective = |x: &[f64]| -> f64 {
            if x.len() != 3 { return f64::INFINITY; }
            let vec = SpiceVector3::new(x[0], x[1], x[2]);
            let result = a * vec;
            vec.dot(&result) // Quadratic form x^T A x
        };
        
        // Use gradient computation to find the minimum (should be at origin)
        let gradient = NumericalDifferentiation::gradient(&objective, &[0.1, 0.1, 0.1], 1e-8).unwrap();
        
        // Gradient at near-zero should be very small
        assert!(gradient[0].abs() < 1e-6);
        assert!(gradient[1].abs() < 1e-6);
        assert!(gradient[2].abs() < 1e-6);
        
        // Test that Newton-Raphson can find the root of the gradient
        let grad_x = |x: f64| -> f64 {
            let obj_at_x = |coords: &[f64]| objective(&[coords[0], 0.1, 0.1]);
            NumericalDifferentiation::forward_difference(&obj_at_x, &[x], 1e-8).unwrap()[0]
        };
        
        let root = OptimizationMethods::newton_raphson(&grad_x, 0.1, 1e-8, 50).unwrap();
        assert!(root.abs() < 1e-6);
    }

    /// Test coordinate transformations with numerical differentiation
    #[test]
    fn test_coordinates_with_numerical_methods() {
        // Test numerical differentiation of coordinate transformations
        let coord_func = |params: &[f64]| -> f64 {
            if params.len() != 3 { return 0.0; }
            let rectangular = SpiceVector3::new(params[0], params[1], params[2]);
            let spherical = rectangular_to_spherical(&rectangular).unwrap();
            spherical.radius // Return radius as a function of rectangular coordinates
        };
        
        // Test at a point where we can verify the gradient analytically
        let point = [3.0, 4.0, 0.0]; // Forms a 3-4-5 right triangle
        let gradient = NumericalDifferentiation::gradient(&coord_func, &point, 1e-8).unwrap();
        
        // The gradient of radius w.r.t. rectangular coordinates should be the unit vector
        let radius = (point[0].powi(2) + point[1].powi(2) + point[2].powi(2)).sqrt();
        let expected_grad_x = point[0] / radius;
        let expected_grad_y = point[1] / radius;
        let expected_grad_z = point[2] / radius;
        
        assert!((gradient[0] - expected_grad_x).abs() < 1e-6);
        assert!((gradient[1] - expected_grad_y).abs() < 1e-6);
        assert!((gradient[2] - expected_grad_z).abs() < 1e-6);
    }

    /// Test curve fitting with foundation vector operations
    #[test]
    fn test_curve_fitting_with_vectors() {
        // Generate test data using vector operations
        let mut x_data = Vec::new();
        let mut y_data = Vec::new();
        
        for i in 0..10 {
            let t = i as f64 * 0.1;
            x_data.push(t);
            
            // Generate y = 2*t^2 + 3*t + 1 with vector-based calculation
            let coeffs = SpiceVector3::new(1.0, 3.0, 2.0); // [constant, linear, quadratic]
            let powers = SpiceVector3::new(1.0, t, t * t);
            y_data.push(coeffs.dot(&powers));
        }
        
        // Fit a polynomial using our curve fitting methods
        let coefficients = CurveFitting::polynomial_least_squares(&x_data, &y_data, 2).unwrap();
        
        // Should recover the original coefficients [1, 3, 2]
        assert!((coefficients[0] - 1.0).abs() < 1e-10);
        assert!((coefficients[1] - 3.0).abs() < 1e-10);
        assert!((coefficients[2] - 2.0).abs() < 1e-10);
        
        // Test correlation coefficient (should be 1.0 for perfect fit)
        let correlation = CurveFitting::correlation_coefficient(&x_data, &y_data).unwrap();
        assert!((correlation - 1.0).abs() < 1e-10);
    }

    /// Test interpolation methods with time-based data
    #[test]
    fn test_interpolation_with_time_data() {
        // Create time-based test data
        let time_points = vec![0.0, 1.0, 2.0, 3.0, 4.0];
        let position_data = vec![0.0, 1.0, 4.0, 9.0, 16.0]; // t^2
        let velocity_data = vec![0.0, 2.0, 4.0, 6.0, 8.0];  // 2*t
        
        // Test Hermite interpolation (includes derivatives)
        let query_time = 2.5;
        let hermite_result = HermiteInterpolation::interpolate(
            &time_points,
            &position_data,
            &velocity_data,
            query_time
        ).unwrap();
        
        // At t=2.5, should have position ≈ 6.25 and velocity ≈ 5.0
        assert!((hermite_result.0 - 6.25).abs() < 1e-10);
        assert!((hermite_result.1 - 5.0).abs() < 1e-10);
        
        // Test Lagrange interpolation (position only)
        let lagrange_result = LagrangeInterpolation::interpolate(
            &time_points,
            &position_data,
            query_time
        ).unwrap();
        
        // Should match Hermite position result
        assert!((lagrange_result - 6.25).abs() < 1e-10);
    }

    /// Test optimization with constraint functions using vectors
    #[test]
    fn test_constrained_optimization() {
        // Minimize ||x||^2 subject to x[0] + x[1] = 1
        // Solution should be x = [0.5, 0.5]
        
        let objective = |x: &[f64]| -> f64 {
            if x.len() != 2 { return f64::INFINITY; }
            let vec = SpiceVector3::new(x[0], x[1], 0.0);
            vec.magnitude_squared()
        };
        
        // Use penalty method: minimize ||x||^2 + penalty*(x[0]+x[1]-1)^2
        let penalized_objective = |x: &[f64]| -> f64 {
            if x.len() != 2 { return f64::INFINITY; }
            let penalty = 1000.0;
            let constraint_violation = x[0] + x[1] - 1.0;
            objective(x) + penalty * constraint_violation.powi(2)
        };
        
        // Use Nelder-Mead to solve
        let initial_simplex = vec![
            vec![0.0, 0.0],
            vec![1.0, 0.0],
            vec![0.0, 1.0]
        ];
        
        let result = OptimizationMethods::nelder_mead(
            penalized_objective,
            &initial_simplex,
            1e-8,
            1000
        ).unwrap();
        
        // Should converge to [0.5, 0.5]
        assert!((result[0] - 0.5).abs() < 1e-3);
        assert!((result[1] - 0.5).abs() < 1e-3);
        
        // Verify constraint satisfaction
        assert!((result[0] + result[1] - 1.0).abs() < 1e-3);
    }

    /// Performance benchmark test to ensure integration doesn't degrade performance
    #[test]
    fn test_performance_integration() {
        use std::time::Instant;
        
        // Test that mathematical operations maintain performance when used together
        let start = Instant::now();
        
        for _ in 0..1000 {
            // Simulate a complex calculation combining multiple modules
            let matrix = SpiceMatrix3x3::identity();
            let vector = SpiceVector3::new(1.0, 2.0, 3.0);
            let transformed = matrix * vector;
            
            // Use optimization to find a root
            let func = |x: f64| x.powi(3) - 2.0;
            let _root = OptimizationMethods::newton_raphson(&func, 1.0, 1e-10, 50).unwrap();
            
            // Evaluate a Chebyshev polynomial
            let coeffs = vec![1.0, 0.5, 0.25];
            let cheb = ChebyshevPolynomials::new(coeffs, (-1.0, 1.0));
            let _value = cheb.evaluate(0.5).unwrap();
            
            // Use the transformed vector result
            assert!(transformed.magnitude() > 0.0);
        }
        
        let duration = start.elapsed();
        
        // Should complete 1000 iterations in reasonable time (< 1 second)
        assert!(duration.as_millis() < 1000, "Performance regression detected: {:?}", duration);
    }

    /// Test mathematical consistency across different precision levels
    #[test]
    fn test_precision_consistency() {
        // Test that different mathematical methods give consistent results
        let test_function = |x: f64| x.powi(3) - 6.0 * x.powi(2) + 11.0 * x - 6.0; // (x-1)(x-2)(x-3)
        
        // This function has roots at x = 1, 2, 3
        let roots = vec![
            OptimizationMethods::newton_raphson(&test_function, 0.8, 1e-12, 100).unwrap(),
            OptimizationMethods::newton_raphson(&test_function, 1.8, 1e-12, 100).unwrap(),
            OptimizationMethods::newton_raphson(&test_function, 2.8, 1e-12, 100).unwrap(),
        ];
        
        // Verify roots are accurate
        assert!((roots[0] - 1.0).abs() < 1e-10);
        assert!((roots[1] - 2.0).abs() < 1e-10);
        assert!((roots[2] - 3.0).abs() < 1e-10);
        
        // Verify that Brent's method gives the same results
        let brent_roots = vec![
            OptimizationMethods::brent_method(&test_function, 0.5, 1.5, 1e-12, 100).unwrap(),
            OptimizationMethods::brent_method(&test_function, 1.5, 2.5, 1e-12, 100).unwrap(),
            OptimizationMethods::brent_method(&test_function, 2.5, 3.5, 1e-12, 100).unwrap(),
        ];
        
        // Newton-Raphson and Brent should agree
        assert!((roots[0] - brent_roots[0]).abs() < 1e-10);
        assert!((roots[1] - brent_roots[1]).abs() < 1e-10);
        assert!((roots[2] - brent_roots[2]).abs() < 1e-10);
    }
}
