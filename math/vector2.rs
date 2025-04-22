// src/math/vector2.rs

#![allow(dead_code)]

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Vector2 {
    pub x: f64,
    pub y: f64,
}

impl Vector2 {
    pub const fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    pub const fn x(&self) -> f64 {
        self.x
    }

    pub const fn y(&self) -> f64 {
        self.y
    }

    pub fn floor_x(&self) -> i64 {
        self.x.floor() as i64
    }

    pub fn floor_y(&self) -> i64 {
        self.y.floor() as i64
    }

    pub fn add(&self, x: f64, y: f64) -> Vector2 {
        Vector2::new(self.x + x, self.y + y)
    }

    pub fn add_vector(&self, other: &Vector2) -> Vector2 {
        self.add(other.x, other.y)
    }

    pub fn subtract(&self, x: f64, y: f64) -> Vector2 {
        self.add(-x, -y)
    }

    pub fn subtract_vector(&self, other: &Vector2) -> Vector2 {
        self.add(-other.x, -other.y)
    }

    pub fn ceil(&self) -> Vector2 {
        Vector2::new(self.x.ceil(), self.y.ceil())
    }

    pub fn floor(&self) -> Vector2 {
        Vector2::new(self.x.floor(), self.y.floor())
    }

    pub fn round(&self) -> Vector2 {
        Vector2::new(self.x.round(), self.y.round())
    }

    pub fn abs(&self) -> Vector2 {
        Vector2::new(self.x.abs(), self.y.abs())
    }

    pub fn multiply(&self, number: f64) -> Vector2 {
        Vector2::new(self.x * number, self.y * number)
    }

    pub fn divide(&self, number: f64) -> Vector2 {
        Vector2::new(self.x / number, self.y / number)
    }

    pub fn distance(&self, pos: &Vector2) -> f64 {
        self.distance_squared(pos).sqrt()
    }

    pub fn distance_squared(&self, pos: &Vector2) -> f64 {
        let dx = self.x - pos.x;
        let dy = self.y - pos.y;
        (dx * dx) + (dy * dy)
    }

    pub fn length(&self) -> f64 {
        self.length_squared().sqrt()
    }

    pub fn length_squared(&self) -> f64 {
        self.x * self.x + self.y * self.y
    }

    pub fn normalize(&self) -> Vector2 {
        let len_sq = self.length_squared();
        if len_sq > 0.0 {
            self.divide(len_sq.sqrt())
        } else {
            Vector2::new(0.0, 0.0)
        }
    }

    pub fn dot(&self, v: &Vector2) -> f64 {
        self.x * v.x + self.y * v.y
    }
}

impl fmt::Display for Vector2 {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Vector2(x={}, y={})", self.x, self.y)
    }
}