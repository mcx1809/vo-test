use std::path::*;
use std::time::SystemTime;

use nalgebra::*;
use opencv::core::*;

use crate::*;

mod images;
mod imu;

use images::*;
use imu::*;

pub struct Source {
    imu_data: ImuData,
    images_reader: ImagesReader,
}

pub struct SourceFrame {
    timestamp: SystemTime,
    linear_acceleration: Vector3<f64>,
    linear_acceleration_cov: Matrix3<f64>,
    angular_velocity: Vector3<f64>,
    angular_velocity_cov: Matrix3<f64>,
    image: Mat,
    camera_param: Matrix3x4<f64>,
}

impl Source {
    pub async fn new<P: AsRef<Path>>(dir: P) -> Result<Self> {
        match ImuData::load(dir.as_ref(), &Vector3::zeros(), &Vector3::zeros()).await {
            Ok(imu_data) => match ImagesReader::open(dir).await {
                Ok(images_reader) => Ok(Self {
                    imu_data,
                    images_reader,
                }),
                Err(err) => Err(err),
            },
            Err(err) => Err(err),
        }
    }

    pub async fn read_next(&mut self) -> Result<SourceFrame> {
        //
        self.imu_data
            .get_transform(&seconds_to_timestamp(0.0), &seconds_to_timestamp(0.0));

        panic!("");
    }
}

#[cfg(test)]
mod test {
    #[async_std::test]
    async fn test() {}
}
