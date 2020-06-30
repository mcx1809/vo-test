use nalgebra::*;

use super::*;
use crate::*;

pub fn slove_displacement(
    camera_param: &Matrix3x4<f64>,
    camera_param_pi: &Matrix4x3<f64>,
    frame_0: &[Vector2<f64>],
    frame_1: &[Vector2<f64>],
) -> Option<Displacement> {
    if frame_0.len() == frame_1.len() {}

    None
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {}
}
