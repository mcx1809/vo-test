use nalgebra::*;
use opencv::{core::*, features2d::*};

use super::*;

pub struct Matcher {
    matcher: Ptr<BFMatcher>,
    prev_computed: Option<(Mat, opencv::core::Vector<KeyPoint>)>,
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
        let train_descriptors = features.descriptors;
        let train_keypoints = features.keypoints;

        let matched_features = {
            let get_vp = |x, y| Vector2::new(x as f64, y as f64);

            let mut matched_features = train_keypoints
                .iter()
                .map(|kp| MatchedFeature {
                    prev_index: u32::MAX,
                    position: get_vp(kp.pt.x, kp.pt.y),
                    match_degree: 0.0,
                })
                .collect::<Vec<MatchedFeature>>();

            if let Some((query_descriptors, query_keypoints)) = self.prev_computed.take() {
                let mut matches = opencv::core::Vector::<DMatch>::new();
                if query_descriptors.cols() == train_descriptors.cols() {
                    self.matcher
                        .train_match(
                            &query_descriptors,
                            &train_descriptors,
                            &mut matches,
                            &no_array().unwrap(),
                        )
                        .unwrap_or_default();
                }

                for m in matches {
                    let query_kp = &query_keypoints.get(m.query_idx as usize).unwrap();
                    let train_kp = &train_keypoints.get(m.train_idx as usize).unwrap();

                    // 考虑匹配的两点距离
                    let query_kp_vp = get_vp(query_kp.pt.x, query_kp.pt.y);
                    let train_kp_vp = get_vp(train_kp.pt.x, train_kp.pt.y);
                    const DISTANCE_THRESHOLD: f64 = 75.0;
                    let v = query_kp_vp - train_kp_vp;
                    if v.dot(&v) <= DISTANCE_THRESHOLD.powi(2) {
                        let mf = &mut matched_features[m.train_idx as usize];
                        mf.prev_index = m.query_idx as u32;
                        // TODO:
                        mf.match_degree = 1.0;
                    }
                }
            }

            matched_features
        };

        self.prev_computed = Some((train_descriptors, train_keypoints));

        matched_features
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {}
}
