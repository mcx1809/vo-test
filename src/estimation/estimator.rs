use nalgebra::*;

use super::*;
use crate::*;

pub struct Estimator {
    camera_matrix: Matrix3<f64>,
}

impl Estimator {
    pub fn new(camera_matrix: Matrix3<f64>) -> Self {
        Self { camera_matrix }
    }

    pub fn test_slove_displacement(&self, tracked: &track::Tracked) -> Result<Displacement> {
        let mut points_0 = vec![];
        let mut points_1 = vec![];
        for i in 0..tracked.points_count() {
            if let Some(p_0) = tracked.get_point(0, i) {
                if let Some(p_1) = tracked.get_point(3, i) {
                    points_0.push(p_0.vp_position);
                    points_1.push(p_1.vp_position);
                }
            }
        }

        slove_displacement(&self.camera_matrix, &points_0, &points_1)
    }
}
