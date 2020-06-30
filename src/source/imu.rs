use std::collections::BTreeMap;
use std::ops::Bound::*;
use std::path::*;
use std::time::{Duration, SystemTime};

use async_std::fs::File;
use async_std::io::BufReader;
use async_std::prelude::*;
use nalgebra::*;
use rand::prelude::*;
use rand_distr::StandardNormal;

use crate::*;

pub struct ImuData {
    positions: BTreeMap<SystemTime, (Vector3<f64>, Quaternion<f64>)>,
}

impl ImuData {
    pub async fn load<P: AsRef<Path>>(
        dir: P,
        p_std_div: &Vector3<f64>,
        d_std_div: &Vector3<f64>,
    ) -> Result<Self> {
        let path = dir.as_ref().join("groundtruth.txt");
        match File::open(path).await {
            Ok(file) => {
                let mut reader = BufReader::new(file);
                let mut positions = BTreeMap::new();

                'a: loop {
                    let mut line = String::new();
                    match reader.read_line(&mut line).await {
                        Ok(len) => {
                            if len > 0 {
                                if line.chars().next().unwrap() != '#' {
                                    let mut time = 0.0;
                                    let mut tv = [0.0; 3];
                                    let mut qv = [0.0; 4];

                                    match line.split_ascii_whitespace().try_fold(0, |i, field| {
                                        if i < 8 {
                                            field
                                                .parse::<f64>()
                                                .map(|v| {
                                                    if i == 0 {
                                                        time = v;
                                                    } else if i <= 3 {
                                                        tv[i - 1] = v;
                                                    } else {
                                                        qv[i - 4] = v;
                                                    }

                                                    i + 1
                                                })
                                                .map_err(|_| Error::from(ErrorKind::InvalidData))
                                        } else {
                                            Ok(i + 1)
                                        }
                                    }) {
                                        Ok(_) => {
                                            tv[0] += thread_rng().sample::<f64, _>(StandardNormal)
                                                * p_std_div[0];
                                            tv[1] += thread_rng().sample::<f64, _>(StandardNormal)
                                                * p_std_div[1];
                                            tv[2] += thread_rng().sample::<f64, _>(StandardNormal)
                                                * p_std_div[2];

                                            let (roll, pitch, yaw) =
                                                UnitQuaternion::from_quaternion(Quaternion::new(
                                                    qv[3], qv[0], qv[1], qv[2],
                                                ))
                                                .euler_angles();
                                            let q = UnitQuaternion::from_euler_angles(
                                                roll + thread_rng()
                                                    .sample::<f64, _>(StandardNormal)
                                                    * d_std_div[0],
                                                pitch
                                                    + thread_rng().sample::<f64, _>(StandardNormal)
                                                        * d_std_div[1],
                                                yaw + thread_rng().sample::<f64, _>(StandardNormal)
                                                    * d_std_div[2],
                                            )
                                            .quaternion()
                                            .clone();

                                            positions.insert(
                                                SystemTime::UNIX_EPOCH
                                                    + Duration::from_secs_f64(time),
                                                (Vector3::from_row_slice(&tv), q),
                                            );
                                        }
                                        Err(err) => return Err(err),
                                    }
                                }
                            } else {
                                break 'a;
                            }
                        }
                        Err(err) => return Err(err),
                    }
                }

                Ok(Self { positions })
            }
            Err(err) => Err(err),
        }
    }

    pub fn get_transform(
        &self,
        time0: &SystemTime,
        time1: &SystemTime,
    ) -> Option<(Vector3<f64>, UnitQuaternion<f64>)> {
        let get_interpolated = |time: &SystemTime| {
            self.positions
                .range((Unbounded, Included(time)))
                .into_iter()
                .next_back()
                .and_then(|p0| {
                    self.positions
                        .range((Included(time), Unbounded))
                        .next()
                        .map(|p1| {
                            let d1 = p1.0.duration_since(*p0.0).unwrap().as_secs_f64();
                            const EPSILON: f64 = 1e-6;
                            if d1.abs() >= EPSILON {
                                let d0 = time.duration_since(*p0.0).unwrap().as_secs_f64();

                                let f1 = d0 / d1;
                                let f0 = 1.0 - f1;

                                ((p0.1).0 * f0 + (p1.1).0 * f1, (p0.1).1.lerp(&(p1.1).1, f1))
                            } else {
                                *p0.1
                            }
                        })
                })
        };

        get_interpolated(time0).and_then(|p0| {
            get_interpolated(time1).and_then(|p1| {
                p0.1.try_inverse().and_then(|p01i| {
                    Some((p1.0 - p0.0, UnitQuaternion::from_quaternion(p1.1 * p01i)))
                })
            })
        })
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[async_std::test]
    async fn test() {
        let imu_data = ImuData::load(
            "data/rgbd_dataset_freiburg1_xyz",
            &Vector3::new(0.05, 0.05, 0.05),
            &Vector3::new(0.001, 0.001, 0.001),
        )
        .await
        .unwrap();

        if let Some(t) = imu_data.get_transform(
            &seconds_to_timestamp(1305031098.6758),
            &seconds_to_timestamp(1305031099.696),
        ) {
            println!("{} {}", t.0, t.1);
        } else {
            println!("none");
        }
    }
}
