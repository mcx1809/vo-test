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
    use super::*;

    #[async_std::test]
    async fn test() {
        let mut times_reader = utils::TimesReader::open("data/00/times.txt").await.unwrap();
        let mut images_reader = utils::ImagesReader::new("data/00/image_0");
        let mut feature_extractor = feature::Extractor::new();
        let mut matcher = feature::Matcher::new();
        let mut tracker = track::Tracker::new(16);
        let tracked_viewer = utils::TrackedViewer::new();

        'a: loop {
            match times_reader.read_next().await {
                Ok(time) => match images_reader.read_next().await {
                    Ok(img) => {
                        let features = feature_extractor.get_features(&img).await;

                        let matched_features = matcher.process(features).await;

                        tracker.update_matched(&time, &matched_features);
                        let tracked = tracker.get_tracked();

                        println!(
                            "points {} frames {}",
                            tracked.points_count(),
                            tracked.frames_count()
                        );

                        tracked_viewer
                            .show_tracked(&img, &tracked, Some(20))
                            .await
                            .unwrap();
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
