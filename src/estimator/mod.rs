use nalgebra::*;

use crate::*;

pub struct Estimator {}

pub struct Displacement {
    position_diff: Vector3<f64>,
    orientation_diff: Quaternion<f64>,
}

impl Estimator {
    pub fn new() -> Self {
        Self {}
    }
}
