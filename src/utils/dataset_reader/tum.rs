use async_trait::async_trait;

use super::*;
use crate::*;

pub struct TumDatasetImuReader {}

#[async_trait]
impl DatasetImuReader for TumDatasetImuReader {
    async fn read_next(&mut self) -> Result<Imu> {
        panic!("");
    }
}

pub struct TumDatasetPoseReader {}

#[async_trait]
impl DatasetPoseReader for TumDatasetPoseReader {
    async fn read_next(&mut self) -> Result<Pose> {
        panic!("");
    }
}

pub struct TumDatasetImageReader {}

#[async_trait]
impl DatasetImageReader for TumDatasetImageReader {
    async fn read_camera_param(&mut self) -> Result<Matrix3x4<f64>> {
        panic!("");
    }

    async fn read_next(&mut self) -> Result<Mat> {
        panic!("");
    }
}

fn get_tum_dateset_readers() -> (
    TumDatasetImuReader,
    TumDatasetPoseReader,
    TumDatasetImageReader,
) {
    panic!("");
}
