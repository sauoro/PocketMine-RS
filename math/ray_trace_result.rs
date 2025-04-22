// src/math/ray_trace_result.rs

#![allow(dead_code)]

use crate::math::{axis_aligned_bb::AxisAlignedBB, facing::Facing, vector3::Vector3};

#[derive(Debug, Clone, Copy)]
pub struct RayTraceResult {
    pub bb: AxisAlignedBB,
    pub hit_face: Facing,
    pub hit_vector: Vector3,
}

impl RayTraceResult {
    pub fn new(bb: AxisAlignedBB, hit_face: Facing, hit_vector: Vector3) -> Self {
        Self { bb, hit_face, hit_vector }
    }

    pub fn bounding_box(&self) -> &AxisAlignedBB {
        &self.bb
    }

    pub fn hit_face(&self) -> Facing {
        self.hit_face
    }

    pub fn hit_vector(&self) -> &Vector3 {
        &self.hit_vector
    }
}
