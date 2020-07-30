pub use std::io::{Error, ErrorKind, Result};

use std::time::{Duration, SystemTime};

pub mod estimation;
pub mod feature;
pub mod source;
pub mod track;
pub mod utils;

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

    #[async_std::test]
    async fn test() {
        let mut times_reader = utils::TimesReader::open("data/00/times.txt").await.unwrap();
        let mut images_reader = utils::ImagesReader::new("data/00/image_0");
        let mut feature_extractor = feature::Extractor::new();
        let mut matcher = feature::Matcher::new();
        let mut tracker = track::Tracker::new(16);
        let tracked_viewer = utils::TrackedViewer::new();
        let camera_param = utils::read_camera_param("data/00/calib.txt")
            .await
            .unwrap()
            .get(0)
            .unwrap()
            .clone();
        // TODO：
        let camera_matrix = Matrix3::from(camera_param.fixed_columns::<U3>(0));
        let estimator = estimation::Estimator::new(camera_matrix);

        'a: loop {
            match times_reader.read_next().await {
                Ok(time) => match images_reader.read_next().await {
                    Ok(img) => {
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

                        match estimator.test_slove_displacement(&tracked) {
                            Ok(d) => {
                                let o = UnitQuaternion::from_quaternion(d.orientation_diff)
                                    .euler_angles();
                                println!("R: {} {} {}", o.0, o.1, o.2);
                                println!(
                                    "t: {} {} {}",
                                    d.position_diff[0], d.position_diff[1], d.position_diff[2]
                                );
                            }
                            Err(_) => println!("Slove failed."),
                        }
                    }
                    Err(_) => {
                        break 'a;
                    }
                },
                Err(_) => {
                    break 'a;
                }
            }
        }
    }
}
