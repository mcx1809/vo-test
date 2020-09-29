use std::time::SystemTime;

use nalgebra::*;
use opencv::core::*;

use crate::*;

//mod imu;

//use imu::*;

pub struct Source<Reader: utils::DatasetReader> {
    dataset_reader: Reader,
    camera_param: Matrix3x4<f64>,
}

pub struct SourceFrame {
    timestamp: SystemTime,
    real_rnt_since_prev: Option<RnT>,
    linear_acceleration: Vector3<f64>,
    linear_acceleration_cov: Matrix3<f64>,
    angular_velocity: Vector3<f64>,
    angular_velocity_cov: Matrix3<f64>,
    camera_param: Matrix3x4<f64>,
    image: Mat,
}

impl<Reader: utils::DatasetReader> Source<Reader> {
    pub async fn new(reader: Reader) -> Result<Self> {
        let mut reader = reader;
        let cam_param = reader.read_camera_param().await?;

        /*match ImuData::load(dir.as_ref(), &Vector3::zeros(), &Vector3::zeros()).await {
            Ok(imu_data) => {
                Ok(images_reader) => {
                    match utils::read_camera_param(dir.as_ref().join("asd")).await {
                        Ok(camera_params) => match camera_params.get(0) {
                            Some(camera_param) => Ok(Self {
                                imu_data,
                                images_reader,
                                camera_param: *camera_param,
                            }),
                            None => Err(Error::from(ErrorKind::NotFound)),
                        },
                        Err(err) => Err(err),
                    }
                    panic!("")
                }
                Err(err) => Err(err),
                panic!("");
            }
            Err(err) => Err(err),
        }*/
        panic!("");
    }

    pub async fn read_next(&mut self) -> Result<SourceFrame> {
        //
        //self.imu_data
        //    .get_transform(&seconds_to_timestamp(0.0), &seconds_to_timestamp(0.0));

        panic!("");
    }
}

#[cfg(test)]
mod test {
    #[async_std::test]
    async fn test() {}
}
