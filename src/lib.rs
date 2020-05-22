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

        'a: loop {
            match times_reader.read_next().await {
                Ok(time) => match images_reader.read_next().await {
                    Ok(img) => {
                        let matched_features = matcher
                            .process(feature_extractor.get_features(&img).await)
                            .await;
                        println!("{}", matched_features.len());
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
