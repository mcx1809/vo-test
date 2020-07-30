use nalgebra::*;

mod estimator;
mod slover;

pub use estimator::*;
pub use slover::*;

pub struct Displacement {
    pub position_diff: Vector3<f64>,
    pub orientation_diff: Quaternion<f64>,
}
