// src/math/vector_math.rs

#![allow(dead_code)]

use crate::math::vector2::Vector2;

pub struct VectorMath;

impl VectorMath {
    pub fn get_direction2d(azimuth_radians: f64) -> Vector2 {
        Vector2::new(azimuth_radians.cos(), azimuth_radians.sin())
    }
}