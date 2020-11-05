use std::path::*;

use async_std::fs::File;
use async_std::io::BufReader;
use async_std::prelude::*;
use futures::future::*;
use opencv::imgcodecs::*;

use super::*;

struct TimesReader {
    reader: BufReader<File>,
}

impl TimesReader {
    async fn open<P: AsRef<Path>>(dir: P, index: u32) -> Result<Self> {
        Ok(Self {
            reader: BufReader::new(
                File::open(
                    dir.as_ref()
                        .join("sequences")
                        .join(format!("{0:>02}", index))
                        .join("times.txt"),
                )
                .await?,
            ),
        })
    }

    async fn read_next(&mut self) -> Result<SystemTime> {
        let mut line = String::new();
        self.reader.read_line(&mut line).await.and_then(|_| {
            line.trim()
                .parse::<f64>()
                .map(|time| seconds_to_timestamp(time))
                .map_err(|_| Error::from(ErrorKind::InvalidData))
        })
    }
}

struct PosesReader {
    reader: BufReader<File>,
}

impl PosesReader {
    async fn open<P: AsRef<Path>>(dir: P, index: u32) -> Result<Self> {
        Ok(Self {
            reader: BufReader::new(
                File::open(
                    dir.as_ref()
                        .join("poses")
                        .join(format!("{0:>02}.txt", index)),
                )
                .await?,
            ),
        })
    }

    async fn read_next(&mut self) -> Result<Pose> {
        let mut line = String::new();
        self.reader.read_line(&mut line).await.and_then(|_| {
            let mut vv = [0.0; 12];
            line.split_ascii_whitespace()
                .try_fold(0, |i, field| {
                    if i < 12 {
                        field
                            .parse::<f64>()
                            .map(|v| {
                                vv[i] = v;
                                i + 1
                            })
                            .map_err(|_| Error::from(ErrorKind::InvalidData))
                    } else {
                        Err(Error::from(ErrorKind::InvalidData))
                    }
                })
                .map(|_| Pose {
                    orientation: *UnitQuaternion::from_matrix(&Matrix3::new(
                        vv[0], vv[1], vv[2], vv[4], vv[5], vv[6], vv[8], vv[9], vv[10],
                    ))
                    .quaternion(),
                    position: Vector3::new(vv[3], vv[7], vv[11]),
                })
        })
    }
}

pub struct KittiPoseSource {
    times_reader: TimesReader,
    poses_reader: PosesReader,
}

impl KittiPoseSource {
    pub async fn open<P: AsRef<Path>>(dir: P, index: u32) -> Result<Self> {
        TimesReader::open(dir.as_ref(), index)
            .try_join(PosesReader::open(dir.as_ref(), index))
            .await
            .map(|(times_reader, poses_reader)| Self {
                times_reader,
                poses_reader,
            })
    }
}

#[async_trait]
impl PoseSource for KittiPoseSource {
    async fn read_next(&mut self) -> Result<(SystemTime, Pose)> {
        let time_fut = self.times_reader.read_next();
        let pose_fut = self.poses_reader.read_next();

        time_fut.try_join(pose_fut).await
    }
}

pub struct KittiCameraSource {
    times_reader: TimesReader,
    dir: PathBuf,
    cam_num: u32,
    frame_index: usize,
}

impl KittiCameraSource {
    pub async fn open<P: AsRef<Path>>(dir: P, index: u32, cam_num: u32) -> Result<Self> {
        Ok(Self {
            times_reader: TimesReader::open(dir.as_ref(), index).await?,
            dir: dir
                .as_ref()
                .join("sequences")
                .join(format!("{0:>02}", index)),
            cam_num,
            frame_index: 0,
        })
    }
}

#[async_trait]
impl CameraSource for KittiCameraSource {
    async fn read_camera_params(&mut self) -> Result<Vec<Matrix3x4<f64>>> {
        let mut reader = BufReader::new(File::open(self.dir.join("calib.txt")).await?);

        // TODO: 读取行数不受限
        let mut params = Vec::new();
        'a: loop {
            let mut line = String::new();
            // TODO: 读取长度不受限
            if reader.read_line(&mut line).await? != 0 {
                let mut vv = [0.0; 12];
                match line.split_ascii_whitespace().try_fold(0, |i, field| {
                    let r = if i >= 1 {
                        field
                            .parse::<f64>()
                            .map(|v| {
                                vv[i - 1] = v;
                                i + 1
                            })
                            .map_err(|_| Error::from(ErrorKind::InvalidData))
                    } else {
                        Ok(i + 1)
                    };

                    if i == 12 {
                        params.push(Matrix3x4::from_row_slice(&vv));
                        Err(Error::from(ErrorKind::Other))
                    } else {
                        r
                    }
                }) {
                    Ok(_) => {
                        return Err(Error::from(ErrorKind::InvalidData));
                    }
                    Err(err) => {
                        if err.kind() != ErrorKind::Other {
                            return Err(err);
                        }
                    }
                }
            } else {
                break 'a;
            }
        }

        Ok(params)
    }

    async fn read_next(&mut self) -> Result<(SystemTime, Vec<Mat>)> {
        let time_fut = self.times_reader.read_next();

        let dir = self.dir.clone();
        let frame_index = self.frame_index;
        let mats_fut = try_join_all((0..self.cam_num).map(|i| {
            let dir = dir.clone();

            async move {
                // TODO: 缓冲优化
                let mut file = File::open(
                    dir.join(format!("image_{}", i))
                        .join(format!("{:06}.png", frame_index)),
                )
                .await?;

                let mut buf = Vec::new();
                file.read_to_end(&mut buf).await?;
                imdecode(
                    &opencv::core::Vector::<u8>::from_iter(buf.into_iter()),
                    IMREAD_GRAYSCALE,
                )
                .map_err(|_| Error::from(ErrorKind::InvalidData))
            }
        }));

        self.frame_index += 1;

        time_fut.try_join(mats_fut).await
    }
}

pub async fn get_kitti_sources<P: AsRef<Path>>(
    dir: P,
    index: u32,
    cam_num: u32,
) -> Result<(KittiPoseSource, KittiCameraSource)> {
    KittiPoseSource::open(dir.as_ref(), index)
        .try_join(KittiCameraSource::open(dir.as_ref(), index, cam_num))
        .await
}

#[cfg(test)]
mod test {
    use opencv::highgui::*;

    use super::*;

    #[async_std::test]
    async fn test_poses_source() -> Result<()> {
        let mut poses_source = KittiPoseSource::open("data/dataset/kitti", 0).await?;

        'a: loop {
            match poses_source.read_next().await {
                Ok((time, pose)) => {
                    let o = UnitQuaternion::from_quaternion(pose.orientation).euler_angles();
                    println!(
                        "time: {}, pose: {} {} {} {} {} {}",
                        timestamp_to_seconds(&time),
                        pose.position.x,
                        pose.position.y,
                        pose.position.z,
                        o.0,
                        o.1,
                        o.2
                    );
                }
                Err(err) => {
                    println!("{}", err);
                    break 'a;
                }
            }
        }

        Ok(())
    }

    #[async_std::test]
    async fn test_camera_source() -> Result<()> {
        let mut camera_source = KittiCameraSource::open("data/dataset/kitti", 0, 2).await?;

        let camera_params = camera_source.read_camera_params().await?;
        camera_params.into_iter().for_each(|p| println!("{}", p));

        'a: loop {
            match camera_source.read_next().await {
                Ok((time, images)) => {
                    println!("time: {}", timestamp_to_seconds(&time));

                    let mut dst = Mat::default().unwrap();
                    hconcat2(&images[0], &images[1], &mut dst).unwrap();
                    imshow("test", &dst).unwrap();

                    wait_key(20).unwrap();
                }
                Err(err) => {
                    println!("{}", err);
                    break 'a;
                }
            }
        }

        Ok(())
    }
}
