// src/math/math.rs

#![allow(dead_code)]

// This struct is empty as the PHP class only contained static methods.
// We can implement these as free functions in Rust.
pub struct Math;

#[inline]
pub fn floor_float(n: f64) -> i64 {
    n.floor() as i64
}

#[inline]
pub fn ceil_float(n: f64) -> i64 {
    n.ceil() as i64
}

pub fn solve_quadratic(a: f64, b: f64, c: f64) -> Result<Vec<f64>, String> {
    if a.abs() < 1e-10 { // Check if 'a' is close to zero
        return Err("Coefficient a cannot be 0!".to_string());
    }
    let discriminant = b * b - 4.0 * a * c;
    if discriminant > 1e-10 { // Two real roots (use epsilon)
        let sqrt_discriminant = discriminant.sqrt();
        Ok(vec![
            (-b + sqrt_discriminant) / (2.0 * a),
            (-b - sqrt_discriminant) / (2.0 * a),
        ])
    } else if discriminant.abs() < 1e-10 { // One real root (use epsilon)
        Ok(vec![-b / (2.0 * a)])
    } else { // No real roots
        Ok(Vec::new())
    }
}