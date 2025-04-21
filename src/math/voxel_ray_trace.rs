// src/math/voxel_ray_trace.rs

#![allow(dead_code)]

use crate::math::vector3::Vector3;

pub struct VoxelRayTrace;

impl VoxelRayTrace {
    pub fn in_direction(start: Vector3, direction_vector: Vector3, max_distance: f64) -> impl Iterator<Item = Vector3> {
        let end = start.add_vector(&direction_vector.multiply(max_distance));
        Self::between_points_internal(start, end)
    }

    pub fn between_points(start: Vector3, end: Vector3) -> Result<impl Iterator<Item = Vector3>, String> {
        let direction_vector = end.subtract_vector(&start).normalize();
        if direction_vector.length_squared() <= 1e-10 {
            return Err("Start and end points are the same, giving a zero direction vector".to_string());
        }
        Ok(Self::between_points_internal(start, end))
    }

    fn between_points_internal(start: Vector3, end: Vector3) -> impl Iterator<Item = Vector3> {
        let direction_vector = end.subtract_vector(&start).normalize();
        let radius = start.distance(&end);

        let mut current_block = start.floor();

        let step_x: f64 = if direction_vector.x > 0.0 { 1.0 } else if direction_vector.x < 0.0 { -1.0 } else { 0.0 };
        let step_y: f64 = if direction_vector.y > 0.0 { 1.0 } else if direction_vector.y < 0.0 { -1.0 } else { 0.0 };
        let step_z: f64 = if direction_vector.z > 0.0 { 1.0 } else if direction_vector.z < 0.0 { -1.0 } else { 0.0 };

        let mut t_max_x = Self::distance_factor_to_boundary(start.x, direction_vector.x);
        let mut t_max_y = Self::distance_factor_to_boundary(start.y, direction_vector.y);
        let mut t_max_z = Self::distance_factor_to_boundary(start.z, direction_vector.z);

        let t_delta_x = if direction_vector.x.abs() < 1e-10 { f64::INFINITY } else { step_x.abs() / direction_vector.x.abs() };
        let t_delta_y = if direction_vector.y.abs() < 1e-10 { f64::INFINITY } else { step_y.abs() / direction_vector.y.abs() };
        let t_delta_z = if direction_vector.z.abs() < 1e-10 { f64::INFINITY } else { step_z.abs() / direction_vector.z.abs() };

        let mut finished = false;

        std::iter::from_fn(move || {
            if finished {
                return None;
            }

            let yielded_block = current_block;

            if t_max_x < t_max_y && t_max_x < t_max_z {
                if t_max_x > radius { finished = true; return Some(yielded_block); }
                current_block = current_block.add(step_x, 0.0, 0.0);
                t_max_x += t_delta_x;
            } else if t_max_y < t_max_z {
                if t_max_y > radius { finished = true; return Some(yielded_block); }
                current_block = current_block.add(0.0, step_y, 0.0);
                t_max_y += t_delta_y;
            } else {
                if t_max_z > radius { finished = true; return Some(yielded_block); }
                if t_delta_z == f64::INFINITY && step_z == 0.0 {
                    if t_max_x >= radius && t_max_y >= radius {
                        finished = true; return Some(yielded_block);
                    } else {
                        finished = true;
                    }
                } else {
                    current_block = current_block.add(0.0, 0.0, step_z);
                    t_max_z += t_delta_z;
                }
            }

            if t_max_x == f64::INFINITY && t_max_y == f64::INFINITY && t_max_z == f64::INFINITY {
                finished = true;
                if !(t_max_x > radius || t_max_y > radius || t_max_z > radius) {
                    return Some(yielded_block);
                } else {
                    return None;
                }
            }

            Some(yielded_block)
        })
    }


    fn distance_factor_to_boundary(s: f64, ds: f64) -> f64 {
        if ds.abs() < 1e-10 {
            return f64::INFINITY;
        }
        if ds < 0.0 {
            let frac = s - s.floor();
            if frac < 1e-10 {
                1.0 / -ds
            } else {
                frac / -ds
            }
        } else {
            let frac = s - s.floor();
            if (1.0 - frac) < 1e-10 {
                1.0 / ds
            } else {
                (1.0 - frac) / ds
            }
        }
    }
}