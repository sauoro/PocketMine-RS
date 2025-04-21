// src/math/axis_aligned_bb.rs

#![allow(dead_code)]

use crate::math::{
    vector3::Vector3,
    facing::Facing,
    axis::Axis,
    ray_trace_result::RayTraceResult
};
use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct AxisAlignedBB {
    pub min_x: f64,
    pub min_y: f64,
    pub min_z: f64,
    pub max_x: f64,
    pub max_y: f64,
    pub max_z: f64,
}

impl AxisAlignedBB {
    pub fn new(min_x: f64, min_y: f64, min_z: f64, max_x: f64, max_y: f64, max_z: f64) -> Self {
        // In Rust, we might prefer returning a Result or panicking. Let's panic for now to match PHP.
        if min_x > max_x { panic!("min_x {} is larger than max_x {}", min_x, max_x); }
        if min_y > max_y { panic!("min_y {} is larger than max_y {}", min_y, max_y); }
        if min_z > max_z { panic!("min_z {} is larger than max_z {}", min_z, max_z); }
        Self { min_x, min_y, min_z, max_x, max_y, max_z }
    }

    pub fn add_coord(&self, x: f64, y: f64, z: f64) -> Self {
        let mut new_bb = *self;
        if x < 0.0 { new_bb.min_x += x; } else if x > 0.0 { new_bb.max_x += x; }
        if y < 0.0 { new_bb.min_y += y; } else if y > 0.0 { new_bb.max_y += y; }
        if z < 0.0 { new_bb.min_z += z; } else if z > 0.0 { new_bb.max_z += z; }
        new_bb
    }

    // Methods returning &mut self are less common in Rust unless part of a builder pattern.
    // We provide both mutable (`expand`) and immutable copy (`expanded_copy`) versions.
    pub fn expand(&mut self, x: f64, y: f64, z: f64) {
        self.min_x -= x;
        self.min_y -= y;
        self.min_z -= z;
        self.max_x += x;
        self.max_y += y;
        self.max_z += z;
    }

    pub fn expanded_copy(&self, x: f64, y: f64, z: f64) -> Self {
        let mut new_bb = *self;
        new_bb.expand(x, y, z);
        new_bb
    }

    pub fn offset(&mut self, x: f64, y: f64, z: f64) {
        self.min_x += x;
        self.min_y += y;
        self.min_z += z;
        self.max_x += x;
        self.max_y += y;
        self.max_z += z;
    }

    pub fn offset_copy(&self, x: f64, y: f64, z: f64) -> Self {
        let mut new_bb = *self;
        new_bb.offset(x, y, z);
        new_bb
    }

    pub fn offset_towards(&mut self, face: Facing, distance: f64) {
        let offset = Facing::get_offset(face);
        self.offset(
            offset[0] as f64 * distance,
            offset[1] as f64 * distance,
            offset[2] as f64 * distance
        );
    }

    pub fn offset_towards_copy(&self, face: Facing, distance: f64) -> Self {
        let mut new_bb = *self;
        new_bb.offset_towards(face, distance);
        new_bb
    }

    pub fn contract(&mut self, x: f64, y: f64, z: f64) {
        self.expand(-x, -y, -z); // Contract is inverse of expand
    }

    pub fn contracted_copy(&self, x: f64, y: f64, z: f64) -> Self {
        self.expanded_copy(-x, -y, -z)
    }

    pub fn extend(&mut self, face: Facing, distance: f64) {
        match face {
            Facing::Down => self.min_y -= distance,
            Facing::Up => self.max_y += distance,
            Facing::North => self.min_z -= distance,
            Facing::South => self.max_z += distance,
            Facing::West => self.min_x -= distance,
            Facing::East => self.max_x += distance,
        }
    }

    pub fn extended_copy(&self, face: Facing, distance: f64) -> Self {
        let mut new_bb = *self;
        new_bb.extend(face, distance);
        new_bb
    }

    pub fn trim(&mut self, face: Facing, distance: f64) {
        self.extend(face, -distance);
    }

    pub fn trimmed_copy(&self, face: Facing, distance: f64) -> Self {
        self.extended_copy(face, -distance)
    }

    pub fn stretch(&mut self, axis: Axis, distance: f64) {
        match axis {
            Axis::Y => { self.min_y -= distance; self.max_y += distance; }
            Axis::Z => { self.min_z -= distance; self.max_z += distance; }
            Axis::X => { self.min_x -= distance; self.max_x += distance; }
        }
    }

    pub fn stretched_copy(&self, axis: Axis, distance: f64) -> Self {
        let mut new_bb = *self;
        new_bb.stretch(axis, distance);
        new_bb
    }

    pub fn squash(&mut self, axis: Axis, distance: f64) {
        self.stretch(axis, -distance);
    }

    pub fn squashed_copy(&self, axis: Axis, distance: f64) -> Self {
        self.stretched_copy(axis, -distance)
    }

    pub fn calculate_x_offset(&self, bb: &AxisAlignedBB, mut x: f64) -> f64 {
        if bb.max_y <= self.min_y || bb.min_y >= self.max_y { return x; }
        if bb.max_z <= self.min_z || bb.min_z >= self.max_z { return x; }

        if x > 0.0 && bb.max_x <= self.min_x {
            let x1 = self.min_x - bb.max_x;
            if x1 < x { x = x1; }
        } else if x < 0.0 && bb.min_x >= self.max_x {
            let x2 = self.max_x - bb.min_x;
            if x2 > x { x = x2; }
        }
        x
    }

    pub fn calculate_y_offset(&self, bb: &AxisAlignedBB, mut y: f64) -> f64 {
        if bb.max_x <= self.min_x || bb.min_x >= self.max_x { return y; }
        if bb.max_z <= self.min_z || bb.min_z >= self.max_z { return y; }

        if y > 0.0 && bb.max_y <= self.min_y {
            let y1 = self.min_y - bb.max_y;
            if y1 < y { y = y1; }
        } else if y < 0.0 && bb.min_y >= self.max_y {
            let y2 = self.max_y - bb.min_y;
            if y2 > y { y = y2; }
        }
        y
    }

    pub fn calculate_z_offset(&self, bb: &AxisAlignedBB, mut z: f64) -> f64 {
        if bb.max_x <= self.min_x || bb.min_x >= self.max_x { return z; }
        if bb.max_y <= self.min_y || bb.min_y >= self.max_y { return z; }

        if z > 0.0 && bb.max_z <= self.min_z {
            let z1 = self.min_z - bb.max_z;
            if z1 < z { z = z1; }
        } else if z < 0.0 && bb.min_z >= self.max_z {
            let z2 = self.max_z - bb.min_z;
            if z2 > z { z = z2; }
        }
        z
    }

    pub fn intersects_with(&self, bb: &AxisAlignedBB, epsilon: f64) -> bool {
        if bb.max_x - self.min_x > epsilon && self.max_x - bb.min_x > epsilon {
            if bb.max_y - self.min_y > epsilon && self.max_y - bb.min_y > epsilon {
                return bb.max_z - self.min_z > epsilon && self.max_z - bb.min_z > epsilon;
            }
        }
        false
    }

    pub fn is_vector_inside(&self, vector: &Vector3) -> bool {
        if vector.x <= self.min_x || vector.x >= self.max_x { return false; }
        if vector.y <= self.min_y || vector.y >= self.max_y { return false; }
        vector.z > self.min_z && vector.z < self.max_z // Note: Original has > and < for Z, <= and >= for X/Y? Assuming typo, using >= <= for all. Let's stick to original for now.
        // Let's correct it to be consistent, likely intended behavior:
        // vector.x >= self.min_x && vector.x <= self.max_x &&
        // vector.y >= self.min_y && vector.y <= self.max_y &&
        // vector.z >= self.min_z && vector.z <= self.max_z
        // Sticking to original translation for now:
        // vector.x >= self.min_x && vector.x <= self.max_x &&
        // vector.y >= self.min_y && vector.y <= self.max_y &&
        // vector.z > self.min_z && vector.z < self.max_z
        // Okay, re-reading PHP: `vector->x <= minX or vector->x >= maxX`. The condition is for *outside*. So inside is:
        // vector.x > self.min_x && vector.x < self.max_x &&
        // vector.y > self.min_y && vector.y < self.max_y &&
        // vector.z > self.min_z && vector.z < self.max_z
        // The PHP code seems inconsistent with the Z axis check. Let's assume strict inequality was intended.
        // vector.x > self.min_x && vector.x < self.max_x &&
        // vector.y > self.min_y && vector.y < self.max_y &&
        // vector.z > self.min_z && vector.z < self.max_z
    }


    pub fn get_average_edge_length(&self) -> f64 {
        (self.get_x_length() + self.get_y_length() + self.get_z_length()) / 3.0
    }

    pub fn get_x_length(&self) -> f64 { self.max_x - self.min_x }
    pub fn get_y_length(&self) -> f64 { self.max_y - self.min_y }
    pub fn get_z_length(&self) -> f64 { self.max_z - self.min_z }

    pub fn is_cube(&self, epsilon: f64) -> bool {
        let x_len = self.get_x_length();
        let y_len = self.get_y_length();
        let z_len = self.get_z_length();
        (x_len - y_len).abs() < epsilon && (y_len - z_len).abs() < epsilon
    }

    pub fn get_volume(&self) -> f64 {
        self.get_x_length() * self.get_y_length() * self.get_z_length()
    }

    pub fn is_vector_in_yz(&self, vector: &Vector3) -> bool {
        vector.y >= self.min_y && vector.y <= self.max_y && vector.z >= self.min_z && vector.z <= self.max_z
    }

    pub fn is_vector_in_xz(&self, vector: &Vector3) -> bool {
        vector.x >= self.min_x && vector.x <= self.max_x && vector.z >= self.min_z && vector.z <= self.max_z
    }

    pub fn is_vector_in_xy(&self, vector: &Vector3) -> bool {
        vector.x >= self.min_x && vector.x <= self.max_x && vector.y >= self.min_y && vector.y <= self.max_y
    }

    pub fn calculate_intercept(&self, pos1: &Vector3, pos2: &Vector3) -> Option<RayTraceResult> {
        let mut v1 = pos1.get_intermediate_with_xvalue(pos2, self.min_x);
        let mut v2 = pos1.get_intermediate_with_xvalue(pos2, self.max_x);
        let mut v3 = pos1.get_intermediate_with_yvalue(pos2, self.min_y);
        let mut v4 = pos1.get_intermediate_with_yvalue(pos2, self.max_y);
        let mut v5 = pos1.get_intermediate_with_zvalue(pos2, self.min_z);
        let mut v6 = pos1.get_intermediate_with_zvalue(pos2, self.max_z);

        if v1.is_some() && !self.is_vector_in_yz(&v1.unwrap()) { v1 = None; }
        if v2.is_some() && !self.is_vector_in_yz(&v2.unwrap()) { v2 = None; }
        if v3.is_some() && !self.is_vector_in_xz(&v3.unwrap()) { v3 = None; }
        if v4.is_some() && !self.is_vector_in_xz(&v4.unwrap()) { v4 = None; }
        if v5.is_some() && !self.is_vector_in_xy(&v5.unwrap()) { v5 = None; }
        if v6.is_some() && !self.is_vector_in_xy(&v6.unwrap()) { v6 = None; }

        let mut closest_vector: Option<Vector3> = None;
        let mut min_distance_sq = f64::MAX;
        let mut hit_face: Option<Facing> = None;

        let candidates = [
            (Facing::West, v1), (Facing::East, v2),
            (Facing::Down, v3), (Facing::Up, v4),
            (Facing::North, v5), (Facing::South, v6)
        ];

        for (face, vector_option) in candidates.iter() {
            if let Some(vector) = vector_option {
                let dist_sq = pos1.distance_squared(vector);
                if dist_sq < min_distance_sq {
                    min_distance_sq = dist_sq;
                    closest_vector = Some(*vector);
                    hit_face = Some(*face);
                }
            }
        }

        if let (Some(vector), Some(face)) = (closest_vector, hit_face) {
            Some(RayTraceResult::new(*self, face, vector))
        } else {
            None
        }
    }

    pub fn one() -> AxisAlignedBB {
        AxisAlignedBB::new(0.0, 0.0, 0.0, 1.0, 1.0, 1.0)
    }
}

impl fmt::Display for AxisAlignedBB {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "AxisAlignedBB({}, {}, {}, {}, {}, {})",
               self.min_x, self.min_y, self.min_z, self.max_x, self.max_y, self.max_z)
    }
}
