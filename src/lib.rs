pub use std::io::{Error, ErrorKind, Result};

use std::time::{Duration, SystemTime};

use nalgebra::*;

pub mod estimation;
pub mod feature;
pub mod source;
pub mod track;
pub mod utils;

pub struct RnT {
    pub position_diff: Vector3<f64>,
    pub orientation_diff: Quaternion<f64>,
}

struct Imu {
    acceleration: Vector3<f64>,
    acceleration_stdev: Vector3<f64>,
    angular_velocity: Vector3<f64>,
    angular_velocity_stdev: Vector3<f64>,
}

struct Pose {
    orientation: Quaternion<f64>,
    position: Vector3<f64>,
}

pub fn timestamp_to_seconds(time: &SystemTime) -> f64 {
    time.duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs_f64()
}

pub fn seconds_to_timestamp(time: f64) -> SystemTime {
    SystemTime::UNIX_EPOCH + Duration::from_secs_f64(time)
}

#[cfg(test)]
mod test {
    use nalgebra::*;

    use super::*;
    use utils::*;

    #[async_std::test]
    async fn test() {
        let mut dataset_reader =
            utils::KittiDatasetReader::open("data/dataset/kitti/sequences/00", 0)
                .await
                .unwrap();
        let mut feature_extractor = feature::Extractor::new();
        let mut matcher = feature::Matcher::new();
        let mut tracker = track::Tracker::new(16);
        let tracked_viewer = utils::TrackedViewer::new();

        // TODO:
        let camera_param = dataset_reader.read_camera_param().await.unwrap();
        let camera_matrix = Matrix3::from(camera_param.fixed_columns::<U3>(0));
        let estimator = estimation::Estimator::new(camera_matrix);

        'a: loop {
            match dataset_reader.read_next().await {
                Ok((time, _, img)) => {
                    let features = feature_extractor.get_features(&img).await;
                    let matched_features = matcher.process(features).await;
                    tracker.update_matched(&time, &matched_features);
                    let tracked = tracker.get_tracked();

                    // println!(
                    //     "points {} frames {}",
                    //     tracked.points_count(),
                    //     tracked.frames_count()
                    // );

                    tracked_viewer
                        .show_tracked(&img, &tracked, Some(20))
                        .await
                        .unwrap();

                    match estimator.test_slove_transform(&tracked) {
                        Ok(transform) => {
                            let o = UnitQuaternion::from_quaternion(transform.orientation_diff)
                                .euler_angles();
                            println!("R: {} {} {}", o.0, o.1, o.2);
                            println!(
                                "t: {} {} {}",
                                transform.position_diff[0],
                                transform.position_diff[1],
                                transform.position_diff[2]
                            );
                        }
                        Err(_) => println!("Slove failed."),
                    }
                }
                Err(_) => {
                    break 'a;
                }
            }
        }
    }
}
