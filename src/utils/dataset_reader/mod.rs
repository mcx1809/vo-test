use std::time::SystemTime;

use async_trait::async_trait;
use nalgebra::*;
use opencv::core::*;

use crate::*;

mod kitti;

pub use kitti::*;

mod tum;

pub use tum::*;

#[async_trait]
pub trait DatasetReader {
    async fn read_camera_param(&mut self) -> Result<Matrix3x4<f64>>;

    async fn read_next(&mut self) -> Result<(SystemTime, Pose, Mat)>;
}
