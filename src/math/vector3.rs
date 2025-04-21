// src/math/vector3.rs

#![allow(dead_code)]

use crate::math::{vector2::Vector2, facing::Facing};
use std::{fmt, ops::{Add, Sub, Mul, Div}};

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector3 {
    pub x: f64,
    pub y: f64,
    pub z: f64,
}

impl Vector3 {
    pub const fn new(x: f64, y: f64, z: f64) -> Self {
        Self { x, y, z }
    }

    pub const fn zero() -> Self {
        Self::new(0.0, 0.0, 0.0)
    }

    pub const fn x(&self) -> f64 {
        self.x
    }

    pub const fn y(&self) -> f64 {
        self.y
    }

    pub const fn z(&self) -> f64 {
        self.z
    }

    pub fn floor_x(&self) -> i64 {
        self.x.floor() as i64
    }

    pub fn floor_y(&self) -> i64 {
        self.y.floor() as i64
    }

    pub fn floor_z(&self) -> i64 {
        self.z.floor() as i64
    }

    pub fn add(&self, x: f64, y: f64, z: f64) -> Vector3 {
        Vector3::new(self.x + x, self.y + y, self.z + z)
    }

    pub fn add_vector(&self, v: &Vector3) -> Vector3 {
        self.add(v.x, v.y, v.z)
    }

    pub fn subtract(&self, x: f64, y: f64, z: f64) -> Vector3 {
        self.add(-x, -y, -z)
    }

    pub fn subtract_vector(&self, v: &Vector3) -> Vector3 {
        self.add(-v.x, -v.y, -v.z)
    }

    pub fn multiply(&self, number: f64) -> Vector3 {
        Vector3::new(self.x * number, self.y * number, self.z * number)
    }

    pub fn divide(&self, number: f64) -> Vector3 {
        Vector3::new(self.x / number, self.y / number, self.z / number)
    }

    pub fn ceil(&self) -> Vector3 {
        Vector3::new(self.x.ceil(), self.y.ceil(), self.z.ceil())
    }

    pub fn floor(&self) -> Vector3 {
        Vector3::new(self.x.floor(), self.y.floor(), self.z.floor())
    }

    pub fn round(&self) -> Vector3 {
        Vector3::new(self.x.round(), self.y.round(), self.z.round())
    }

    pub fn round_prec(&self, precision: u32) -> Vector3 {
        let factor = 10.0_f64.powi(precision as i32);
        Vector3::new(
            (self.x * factor).round() / factor,
            (self.y * factor).round() / factor,
            (self.z * factor).round() / factor,
        )
    }

    pub fn abs(&self) -> Vector3 {
        Vector3::new(self.x.abs(), self.y.abs(), self.z.abs())
    }

    pub fn get_side(&self, side: Facing, step: i64) -> Vector3 {
        let offset = Facing::get_offset(side);
        self.add(
            (offset[0] as i64 * step) as f64,
            (offset[1] as i64 * step) as f64,
            (offset[2] as i64 * step) as f64,
        )
    }

    pub fn down(&self, step: i64) -> Vector3 {
        self.get_side(Facing::Down, step)
    }

    pub fn up(&self, step: i64) -> Vector3 {
        self.get_side(Facing::Up, step)
    }

    pub fn north(&self, step: i64) -> Vector3 {
        self.get_side(Facing::North, step)
    }

    pub fn south(&self, step: i64) -> Vector3 {
        self.get_side(Facing::South, step)
    }

    pub fn west(&self, step: i64) -> Vector3 {
        self.get_side(Facing::West, step)
    }

    pub fn east(&self, step: i64) -> Vector3 {
        self.get_side(Facing::East, step)
    }

    pub fn sides(&self, step: i64) -> impl Iterator<Item = (Facing, Vector3)> + '_ {
        Facing::ALL.iter().map(move |&facing| (facing, self.get_side(facing, step)))
    }

    pub fn sides_array(&self, step: i64) -> [(Facing, Vector3); 6] {
        [
            (Facing::Down, self.down(step)),
            (Facing::Up, self.up(step)),
            (Facing::North, self.north(step)),
            (Facing::South, self.south(step)),
            (Facing::West, self.west(step)),
            (Facing::East, self.east(step)),
        ]
    }

    pub fn sides_around_axis(&self, axis: crate::math::axis::Axis, step: i64) -> impl Iterator<Item = (Facing, Vector3)> + '_ {
        Facing::ALL.iter().filter(move |&&facing| Facing::axis(facing) != axis)
            .map(move |&facing| (facing, self.get_side(facing, step)))
    }

    pub fn as_vector3(&self) -> Vector3 {
        *self // Since Vector3 is Copy
    }

    pub fn distance(&self, pos: &Vector3) -> f64 {
        self.distance_squared(pos).sqrt()
    }

    pub fn distance_squared(&self, pos: &Vector3) -> f64 {
        let dx = self.x - pos.x;
        let dy = self.y - pos.y;
        let dz = self.z - pos.z;
        (dx * dx) + (dy * dy) + (dz * dz)
    }

    pub fn max_plain_distance(&self, other: &Vector3) -> f64 {
        f64::max((self.x - other.x).abs(), (self.z - other.z).abs())
    }

    pub fn max_plain_distance_vec2(&self, other: &Vector2) -> f64 {
        f64::max((self.x - other.x).abs(), (self.z - other.y).abs())
    }

    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn length_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y + self.z * self.z
    }

    pub fn normalize(&self) -> Vector3 {
        let len_sq = self.length_squared();
        if len_sq > 1e-10 { // Use epsilon for float comparison
            self.divide(len_sq.sqrt())
        } else {
            Vector3::zero()
        }
    }

    pub fn dot(&self, v: &Vector3) -> f64 {
        self.x * v.x + self.y * v.y + self.z * v.z
    }

    pub fn cross(&self, v: &Vector3) -> Vector3 {
        Vector3::new(
            self.y * v.z - self.z * v.y,
            self.z * v.x - self.x * v.z,
            self.x * v.y - self.y * v.x,
        )
    }

    pub fn equals(&self, v: &Vector3) -> bool {
        // Use epsilon for float comparison
        (self.x - v.x).abs() < 1e-10 &&
            (self.y - v.y).abs() < 1e-10 &&
            (self.z - v.z).abs() < 1e-10
    }

    pub fn get_intermediate_with_xvalue(&self, v: &Vector3, x: f64) -> Option<Vector3> {
        let x_diff = v.x - self.x;
        if (x_diff * x_diff) < 1e-10 {
            return None;
        }
        let f = (x - self.x) / x_diff;
        if !(0.0..=1.0).contains(&f) {
            None
        } else {
            Some(Vector3::new(
                x,
                self.y + (v.y - self.y) * f,
                self.z + (v.z - self.z) * f,
            ))
        }
    }

    pub fn get_intermediate_with_yvalue(&self, v: &Vector3, y: f64) -> Option<Vector3> {
        let y_diff = v.y - self.y;
        if (y_diff * y_diff) < 1e-10 {
            return None;
        }
        let f = (y - self.y) / y_diff;
        if !(0.0..=1.0).contains(&f) {
            None
        } else {
            Some(Vector3::new(
                self.x + (v.x - self.x) * f,
                y,
                self.z + (v.z - self.z) * f,
            ))
        }
    }

    pub fn get_intermediate_with_zvalue(&self, v: &Vector3, z: f64) -> Option<Vector3> {
        let z_diff = v.z - self.z;
        if (z_diff * z_diff) < 1e-10 {
            return None;
        }
        let f = (z - self.z) / z_diff;
        if !(0.0..=1.0).contains(&f) {
            None
        } else {
            Some(Vector3::new(
                self.x + (v.x - self.x) * f,
                self.y + (v.y - self.y) * f,
                z,
            ))
        }
    }

    pub fn with_components(&self, x: Option<f64>, y: Option<f64>, z: Option<f64>) -> Vector3 {
        Vector3::new(
            x.unwrap_or(self.x),
            y.unwrap_or(self.y),
            z.unwrap_or(self.z),
        )
    }

    pub fn max_components(vectors: &[Vector3]) -> Option<Vector3> {
        if vectors.is_empty() {
            return None;
        }
        let mut max_x = vectors[0].x;
        let mut max_y = vectors[0].y;
        let mut max_z = vectors[0].z;
        for v in vectors.iter().skip(1) {
            max_x = f64::max(max_x, v.x);
            max_y = f64::max(max_y, v.y);
            max_z = f64::max(max_z, v.z);
        }
        Some(Vector3::new(max_x, max_y, max_z))
    }

    pub fn min_components(vectors: &[Vector3]) -> Option<Vector3> {
        if vectors.is_empty() {
            return None;
        }
        let mut min_x = vectors[0].x;
        let mut min_y = vectors[0].y;
        let mut min_z = vectors[0].z;
        for v in vectors.iter().skip(1) {
            min_x = f64::min(min_x, v.x);
            min_y = f64::min(min_y, v.y);
            min_z = f64::min(min_z, v.z);
        }
        Some(Vector3::new(min_x, min_y, min_z))
    }

    pub fn sum(vectors: &[Vector3]) -> Vector3 {
        let mut sum_x = 0.0;
        let mut sum_y = 0.0;
        let mut sum_z = 0.0;
        for v in vectors {
            sum_x += v.x;
            sum_y += v.y;
            sum_z += v.z;
        }
        Vector3::new(sum_x, sum_y, sum_z)
    }
}

impl fmt::Display for Vector3 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Vector3(x={}, y={}, z={})", self.x, self.y, self.z)
    }
}

// --- Operator Overloads for convenience ---
impl Add for Vector3 {
    type Output = Self;
    fn add(self, other: Self) -> Self {
        self.add_vector(&other)
    }
}
impl Sub for Vector3 {
    type Output = Self;
    fn sub(self, other: Self) -> Self {
        self.subtract_vector(&other)
    }
}
impl Mul<f64> for Vector3 {
    type Output = Self;
    fn mul(self, rhs: f64) -> Self {
        self.multiply(rhs)
    }
}
impl Div<f64> for Vector3 {
    type Output = Self;
    fn div(self, rhs: f64) -> Self {
        self.divide(rhs)
    }
}
