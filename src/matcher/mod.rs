use nalgebra::*;
use opencv::{core::*, features2d::*};

use crate::*;

pub struct Matcher {
    matcher: Ptr<BFMatcher>,
    prev_computed: Option<Mat>,
}

#[derive(Clone)]
pub struct MatchedFeature {
    pub prev_index: u32,
    pub position: Vector2<f64>,
    pub match_degree: f64,
}

impl Matcher {
    pub fn new() -> Self {
        Self {
            matcher: BFMatcher::create(NORM_HAMMING, true).unwrap(),
            prev_computed: None,
        }
    }

    pub async fn process(&mut self, features: Features) -> Vec<MatchedFeature> {
        if let Some(query_descriptors) = self.prev_computed.take() {
            let matched_features = {
                let mut matches = opencv::core::Vector::<DMatch>::new();
                self.matcher
                    .train_match(
                        &query_descriptors,
                        &features.descriptors,
                        &mut matches,
                        &no_array().unwrap(),
                    )
                    .unwrap();

                {
                    let mut matched_features = Vec::<MatchedFeature>::new();

                    matched_features.resize(matches.len(), MatchedFeature::default());
                    for i in 0..matched_features.len() {
                        let m = &matches.get(i).unwrap();
                        let kp = &features.keypoints.get(m.train_idx as usize).unwrap();
                        let mf = &mut matched_features[i];

                        mf.position = Vector2::new(
                            (kp.pt.x as f64) - (features.img_cols as f64) / 2.0,
                            -((kp.pt.y as f64) - (features.img_rows as f64) / 2.0),
                        );

                        // TODO:
                        mf.match_degree = 1.0;

                        mf.prev_index = m.query_idx as u32;
                    }

                    matched_features
                }
            };

            self.prev_computed = Some(features.descriptors);

            matched_features
        } else {
            self.prev_computed = Some(features.descriptors);

            {
                let mut matched_features =
                    vec![MatchedFeature::default(); features.keypoints.len()];

                for i in matched_features.iter_mut().zip(features.keypoints.iter()) {
                    let kp = i.1;
                    let mf = i.0;

                    mf.position = Vector2::new(
                        (kp.pt.x as f64) - (features.img_cols as f64) / 2.0,
                        -((kp.pt.y as f64) - (features.img_rows as f64) / 2.0),
                    );
                }

                matched_features
            }
        }
    }
}

impl MatchedFeature {
    fn default() -> Self {
        Self {
            prev_index: 0,
            position: Vector2::new(0.0, 0.0),
            match_degree: 0.0,
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {}
}
