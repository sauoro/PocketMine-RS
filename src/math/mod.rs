// src/math/mod.rs

pub mod axis;
pub mod axis_aligned_bb;
pub mod facing;
pub mod math;
pub mod matrix;
pub mod ray_trace_result;
pub mod vector2;
pub mod vector3;
pub mod vector_math;
pub mod voxel_ray_trace;

// Re-export commonly used types
pub use axis::Axis;
pub use axis_aligned_bb::AxisAlignedBB;
pub use facing::Facing;
pub use math::Math; // Although Math struct is empty, keep the module
pub use matrix::Matrix;
pub use ray_trace_result::RayTraceResult;
pub use vector2::Vector2;
pub use vector3::Vector3;
pub use vector_math::VectorMath;
pub use voxel_ray_trace::VoxelRayTrace;