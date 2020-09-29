use std::path::*;

use async_std::fs::File;
use async_std::io::BufReader;
use async_std::prelude::*;
use async_trait::async_trait;
use nalgebra::*;
use opencv::core::*;
use opencv::imgcodecs::*;

use super::DatasetReader;
use crate::*;

struct TimesReader {
    reader: BufReader<File>,
}

impl TimesReader {
    pub async fn read_next(&mut self) -> Result<SystemTime> {
        let mut line = String::new();
        self.reader.read_line(&mut line).await.and_then(|_| {
            line.trim()
                .parse::<f64>()
                .map(|time| seconds_to_timestamp(time))
                .map_err(|_| Error::from(ErrorKind::InvalidData))
        })
    }
}

struct PoseReader {
    reader: BufReader<File>,
}

impl PoseReader {
    pub async fn read_next(&mut self) -> Result<Pose> {
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

struct ImagesReader {
    index: u32,
    dir: PathBuf,
    cam_index: u32,
}

impl ImagesReader {
    async fn read_camera_params(&self) -> Result<Vec<Matrix3x4<f64>>> {
        match File::open(self.dir.join("..").join("calib.txt")).await {
            Ok(file) => {
                let mut reader = BufReader::new(file);

                // TODO: 读取行数不受限
                let mut params = Vec::new();
                'a: loop {
                    let mut line = String::new();
                    // TODO: 读取长度不受限
                    match reader.read_line(&mut line).await {
                        Ok(len) => {
                            if len != 0 {
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
                        Err(err) => {
                            return Err(err);
                        }
                    }
                }

                Ok(params)
            }
            Err(err) => Err(err),
        }
    }

    async fn read_camera_param(&self) -> Result<Matrix3x4<f64>> {
        self.read_camera_params().await.and_then(|params| {
            if let Some(param) = params.get(self.cam_index as usize) {
                Ok(*param)
            } else {
                Err(Error::from(ErrorKind::NotFound))
            }
        })
    }

    async fn read_next(&mut self) -> Result<Mat> {
        // TODO: 缓冲优化

        match File::open(self.dir.join(format!("{:06}.png", self.index))).await {
            Ok(mut file) => {
                let mut buf = vec![];
                match file.read_to_end(&mut buf).await {
                    Ok(_) => {
                        match imdecode(
                            &opencv::core::Vector::<u8>::from_iter(buf.into_iter()),
                            IMREAD_GRAYSCALE,
                        ) {
                            Ok(img) => {
                                self.index += 1;
                                Ok(img)
                            }
                            Err(_) => Err(Error::from(ErrorKind::InvalidData)),
                        }
                    }
                    Err(err) => Err(err),
                }
            }
            Err(err) => Err(err),
        }
    }
}

pub struct KittiDatasetReader {
    times_reader: Option<Box<TimesReader>>,
    poses_reader: Option<Box<PoseReader>>,
    images_reader: Option<Box<ImagesReader>>,
}

impl KittiDatasetReader {
    pub async fn open<P: AsRef<Path>>(dir: P, cam_index: u32) -> Result<Self> {
        match File::open(dir.as_ref().join("times.txt")).await {
            Ok(times_file) => {
                if let Some(poses_file_name) = dir.as_ref().file_name() {
                    let mut poses_file_path = dir
                        .as_ref()
                        .join("..")
                        .join("..")
                        .join("poses")
                        .join(poses_file_name);
                    poses_file_path.set_extension("txt");
                    match File::open(poses_file_path).await {
                        Ok(poses_file) => Ok(Self {
                            times_reader: Some(Box::new(TimesReader {
                                reader: BufReader::new(times_file),
                            })),
                            poses_reader: Some(Box::new(PoseReader {
                                reader: BufReader::new(poses_file),
                            })),
                            images_reader: Some(Box::new(ImagesReader {
                                index: 0,
                                dir: dir
                                    .as_ref()
                                    .join(format!("image_{}", cam_index))
                                    .to_path_buf(),
                                cam_index,
                            })),
                        }),
                        Err(err) => Err(err),
                    }
                } else {
                    Err(Error::from(ErrorKind::NotFound))
                }
            }
            Err(err) => Err(err),
        }
    }
}

#[async_trait]
impl DatasetReader for KittiDatasetReader {
    async fn read_camera_param(&mut self) -> Result<Matrix3x4<f64>> {
        if let Some(images_reader) = &self.images_reader {
            images_reader.read_camera_param().await
        } else {
            panic!("Logic error.");
        }
    }

    async fn read_next(&mut self) -> Result<(SystemTime, Pose, Mat)> {
        let mut times_reader = self.times_reader.take().unwrap();
        let mut poses_reader = self.poses_reader.take().unwrap();
        let mut images_reader = self.images_reader.take().unwrap();

        let r = times_reader
            .read_next()
            .try_join(poses_reader.read_next())
            .try_join(images_reader.read_next())
            .await
            .map(|((time, pose), image)| (time, pose, image));

        self.times_reader = Some(times_reader);
        self.poses_reader = Some(poses_reader);
        self.images_reader = Some(images_reader);

        r
    }
}

#[cfg(test)]
mod test {
    use opencv::highgui::*;

    use super::*;

    #[async_std::test]
    async fn test() {
        let mut reader = KittiDatasetReader::open("data/dataset/kitti/sequences/00", 0)
            .await
            .unwrap();

        let cam_param = reader.read_camera_param().await.unwrap();
        println!("camera param:");
        println!("{}", cam_param);

        let mut start_t: Option<SystemTime> = None;

        'a: loop {
            match reader.read_next().await {
                Ok((time, pose, img)) => {
                    let t = timestamp_to_seconds(&time);
                    println!("time:{}", t);

                    if let Some(start_t) = start_t.clone() {
                        let real_t = std::time::SystemTime::now()
                            .duration_since(start_t)
                            .unwrap()
                            .as_secs_f64();

                        if t > real_t {
                            wait_key(((t - real_t) * 1000.0) as i32).unwrap();
                        }
                    } else {
                        start_t = Some(SystemTime::now());
                    }

                    let o = UnitQuaternion::from_quaternion(pose.orientation).euler_angles();
                    println!("orientation: {} {} {}", o.0, o.1, o.2);
                    println!(
                        "pose:{} {} {}",
                        pose.position.x, pose.position.y, pose.position.z
                    );

                    imshow("vo-test", &img).unwrap();
                }
                Err(err) => {
                    println!("{}", err);
                    break 'a;
                }
            }
        }
    }
}
