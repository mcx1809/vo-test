use nalgebra::*;
use opencv::{calib3d::*, core::*, types::*};

use super::*;
use crate::*;

pub fn slove_transform(
    camera_matrix: &Matrix3<f64>,
    points_0: &[Vector2<f64>],
    points_1: &[Vector2<f64>],
) -> Result<RnT> {
    if points_0.len() == points_1.len() {
        if points_0.len() >= 5 {
            let trans_points = |points: &[Vector2<f64>]| {
                points
                    .clone()
                    .into_iter()
                    .map(|p| Point2d::new(p[0], p[1]))
                    .collect::<VectorOfPoint2d>()
            };
            let points_0 = trans_points(points_0);
            let points_1 = trans_points(points_1);

            let cam_mat = {
                let mut mat = Mat::zeros(3, 3, CV_64F).unwrap().to_mat().unwrap();
                *mat.at_2d_mut(0, 0).unwrap() = camera_matrix[(0, 0)];
                *mat.at_2d_mut(0, 1).unwrap() = camera_matrix[(0, 1)];
                *mat.at_2d_mut(0, 2).unwrap() = camera_matrix[(0, 2)];
                *mat.at_2d_mut(1, 0).unwrap() = camera_matrix[(1, 0)];
                *mat.at_2d_mut(1, 1).unwrap() = camera_matrix[(1, 1)];
                *mat.at_2d_mut(1, 2).unwrap() = camera_matrix[(1, 2)];
                *mat.at_2d_mut(2, 0).unwrap() = camera_matrix[(2, 0)];
                *mat.at_2d_mut(2, 1).unwrap() = camera_matrix[(2, 1)];
                *mat.at_2d_mut(2, 2).unwrap() = camera_matrix[(2, 2)];
                mat
            };

            return find_essential_mat_matrix(
                &points_0,
                &points_1,
                &cam_mat,
                RANSAC,
                0.999,
                1.0,
                &mut no_array().unwrap(),
            )
            .map_err(|_| Error::from(ErrorKind::Other))
            .and_then(|e| {
                let mut r = Mat::default().unwrap();
                let mut t = Mat::default().unwrap();

                recover_pose_camera(
                    &e,
                    &points_0,
                    &points_1,
                    &cam_mat,
                    &mut r,
                    &mut t,
                    &mut no_array().unwrap(),
                )
                .map(|_| RnT {
                    position_diff: Vector3::new(
                        *t.at(0).unwrap(),
                        *t.at(1).unwrap(),
                        *t.at(2).unwrap(),
                    ),
                    orientation_diff: *UnitQuaternion::from_matrix(&Matrix3::new(
                        *r.at_2d(0, 0).unwrap(),
                        *r.at_2d(0, 1).unwrap(),
                        *r.at_2d(0, 2).unwrap(),
                        *r.at_2d(1, 0).unwrap(),
                        *r.at_2d(1, 1).unwrap(),
                        *r.at_2d(1, 2).unwrap(),
                        *r.at_2d(2, 0).unwrap(),
                        *r.at_2d(2, 1).unwrap(),
                        *r.at_2d(2, 2).unwrap(),
                    ))
                    .quaternion(),
                })
                .map_err(|_| Error::from(ErrorKind::Other))
            });
        }
    }

    Err(Error::from(ErrorKind::InvalidInput))
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {}
}
