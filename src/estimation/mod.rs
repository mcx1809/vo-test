use nalgebra::*;

mod estimator;
mod slover;

pub use estimator::*;
pub use slover::*;

pub struct Displacement {
    position_diff: Vector3<f64>,
    orientation_diff: Quaternion<f64>,
}
