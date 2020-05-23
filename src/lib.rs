pub use std::io::{Error, ErrorKind, Result};

mod estimator;
mod feature_extractor;
mod matcher;
mod tracker;
mod utils;

pub use estimator::*;
pub use feature_extractor::*;
pub use matcher::*;
pub use tracker::*;
pub use utils::*;

#[cfg(test)]
mod test {
    use super::*;

    #[async_std::test]
    async fn test() {
        let mut times_reader = TimesReader::open("data/00/times.txt").await.unwrap();
        let mut images_reader = ImagesReader::new("data/00/image_0");
        let mut feature_extractor = FeatureExtractor::new();
        let mut matcher = Matcher::new();
        let mut tracker = Tracker::new(4);

        'a: loop {
            match times_reader.read_next().await {
                Ok(time) => match images_reader.read_next().await {
                    Ok(img) => {
                        let features = feature_extractor.get_features(&img).await;

                        let matched_features = matcher.process(features).await;

                        tracker.update_matched(&time, &matched_features);
                        let tracked = tracker.get_tracked();

                        println!("{} {}", tracked.frames_count(), tracked.points_count());
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
