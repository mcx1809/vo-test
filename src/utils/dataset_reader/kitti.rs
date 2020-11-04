use std::path::*;

use async_std::fs::File;
use async_std::io::BufReader;
use async_std::prelude::*;
use async_trait::async_trait;
use chrono::prelude::*;
use nalgebra::*;
use opencv::core::*;
use opencv::imgcodecs::*;

use super::*;
use crate::*;

struct KittiDatasetOxtsReader {
    times_reader: BufReader<File>,
    dir: PathBuf,
    
    index: u32,
}

impl KittiDatasetOxtsReader {
    async fn open<P: AsRef<Path>>(dir: P) -> Result<Self> {
        let oxts_dir = dir.as_ref().join("oxts");

        Ok(Self {
            times_reader: BufReader::new(File::open(oxts_dir.join("timestamps.txt")).await?),
            dir: oxts_dir.join("data"),
            index: 0,
        })
    }

    async fn read_next(&mut self) -> Result<(SystemTime, Imu, Pose)> {
        let mut line = String::new();
        // TODO: 读取长度不受限
        let time = self.times_reader.read_line(&mut line).await.and_then(|r| {
            if r > 0 {
                Ok(SystemTime::from(
                    DateTime::parse_from_str(line.trim_end(), "%Y-%m-%d %H:%M:%S%.f")
                        .map_err(|_| Error::from(ErrorKind::InvalidData))?,
                ))
            } else {
                Err(Error::from(ErrorKind::NotFound))
            }
        })?;

        let mut line = String::new();
        let mut reader =
            BufReader::new(File::open(self.dir.join(format!("{:010}.txt", self.index))).await?);
        let sp = reader
            .read_line(&mut line)
            .await
            .map(|_| line.split_ascii_whitespace().collect::<Vec<&str>>())?;
        let imu = Imu {
            acceleration: Vector3::new(
                sp[0]
                    .parse::<f64>()
                    .map_err(|_| Error::from(ErrorKind::InvalidData))?,
                sp[0].parse::<f64>().unwrap(),
                sp[0].parse::<f64>().unwrap(),
            ),
            acceleration_stdev: Vector3::new(
                sp[0].parse::<f64>().unwrap(),
                sp[0].parse::<f64>().unwrap(),
                sp[0].parse::<f64>().unwrap(),
            ),
            angular_velocity: Vector3::new(
                sp[0].parse::<f64>().unwrap(),
                sp[0].parse::<f64>().unwrap(),
                sp[0].parse::<f64>().unwrap(),
            ),
            angular_velocity_stdev: Vector3::new(
                sp[0].parse::<f64>().unwrap(),
                sp[0].parse::<f64>().unwrap(),
                sp[0].parse::<f64>().unwrap(),
            ),
        };

        self.index += 1;

        panic!("");
    }
}

pub struct KittiDatasetImuReader {
    reader: BufReader<File>,
}

#[async_trait]
impl DatasetImuReader for KittiDatasetImuReader {
    async fn read_next(&mut self) -> Result<Imu> {
        panic!("");
    }
}

pub struct KittiDatasetPoseReader {}

#[async_trait]
impl DatasetPoseReader for KittiDatasetPoseReader {
    async fn read_next(&mut self) -> Result<Pose> {
        panic!("");
    }
}

pub struct KittiDatasetImageReader {}

#[async_trait]
impl DatasetImageReader for KittiDatasetImageReader {
    async fn read_camera_param(&mut self) -> Result<Matrix3x4<f64>> {
        panic!("");
    }

    async fn read_next(&mut self) -> Result<Mat> {
        panic!("");
    }
}

fn get_kitti_dateset_readers<P: AsRef<Path>>(
    dir: P,
    cam_index: u32,
) -> Result<(
    KittiDatasetImuReader,
    KittiDatasetPoseReader,
    KittiDatasetImageReader,
)> {
    //let imu_reader =File::open(dir.as_ref().join("oxts").join(""))

    panic!("");
}
