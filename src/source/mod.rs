use std::time::SystemTime;

use async_trait::async_trait;
use nalgebra::*;
use opencv::core::*;

use crate::*;

mod kitti;

pub use kitti::*;

#[async_trait]
pub trait CameraSource {
    async fn read_camera_params(&mut self) -> Result<Vec<Matrix3x4<f64>>>;

    async fn read_next(&mut self) -> Result<(SystemTime, Vec<Mat>)>;
}

#[async_trait]
pub trait ImuSource {
    async fn read_next(&mut self) -> Result<(SystemTime, Imu)>;
}

#[async_trait]
pub trait PoseSource {
    async fn read_next(&mut self) -> Result<(SystemTime, Pose)>;
}

#[cfg(test)]
mod test {
    #[async_std::test]
    async fn test() {}
}
