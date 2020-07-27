use nalgebra::*;
use opencv::calib3d::*;

use super::*;
use crate::*;

pub fn slove_displacement(
    camera_param: &Matrix3x4<f64>,
    points_0: &[Vector2<f64>],
    points_1: &[Vector2<f64>],
) -> Option<Displacement> {
    if points_0.len() == points_1.len() {
        if points_0.len() >= 5 {

            //find_essential_mat_matrix(points1, points2, camera_matrix, method, prob, threshold, mask)
        }
    }

    None
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {}
}
