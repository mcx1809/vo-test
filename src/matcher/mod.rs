use nalgebra::*;
use opencv::{core::*, features2d::*};
use std::collections::BTreeMap;

use crate::*;

pub struct Matcher {
    matcher: Ptr<BFMatcher>,
    prev_computed: Option<(Mat, opencv::core::Vector<KeyPoint>, Vec<usize>)>,
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
        let img_xo = (features.img_cols as f64) / 2.0;
        let img_yo = (features.img_rows as f64) / 2.0;

        let get_vp = |x, y| Vector2::new(x as f64 - img_xo, -(y as f64 - img_yo));

        if let Some((query_descriptors, query_keypoints, query_matched_indexes)) =
            self.prev_computed.take()
        {
            let (matched_features, matched_indexes) = {
                let (matches, matched_trains) = {
                    let mut matches_raw = opencv::core::Vector::<DMatch>::new();
                    if query_descriptors.cols() == train_descriptors.cols() {
                        self.matcher
                            .train_match(
                                &query_descriptors,
                                &train_descriptors,
                                &mut matches_raw,
                                &no_array().unwrap(),
                            )
                            .unwrap_or_default();
                    }

                    let mut matches = opencv::core::Vector::<DMatch>::new();
                    let mut matched_trains = BTreeMap::new();
                    for m in matches_raw {
                        let query_kp = &query_keypoints.get(m.query_idx as usize).unwrap();
                        let train_kp = &train_keypoints.get(m.train_idx as usize).unwrap();

                        // 考虑匹配的两点距离
                        let query_kp_vp = get_vp(query_kp.pt.x, query_kp.pt.y);
                        let train_kp_vp = get_vp(train_kp.pt.x, train_kp.pt.y);
                        // TODO:
                        const DISTANCE2_THRESHOLD: f64 = 50.0 * 50.0;
                        let v = query_kp_vp - train_kp_vp;
                        if v.dot(&v) <= DISTANCE2_THRESHOLD {
                            matches.push(m);
                            matched_trains.insert(m.train_idx, matches.len() - 1);
                        }
                    }

                    (matches, matched_trains)
                };

                {
                    let mut matched_features = vec![MatchedFeature::default(); matches.len()];
                    for i in matched_features.iter_mut().zip(matches.iter()) {
                        let m = i.1;
                        let kp = &train_keypoints.get(m.train_idx as usize).unwrap();
                        let mf = i.0;

                        mf.position = get_vp(kp.pt.x, kp.pt.y);
                        // TODO:
                        mf.match_degree = 1.0;
                        mf.prev_index =
                            *query_matched_indexes.get(m.query_idx as usize).unwrap() as u32;
                    }

                    let matched_indexes = (0usize..train_keypoints.len())
                        .into_iter()
                        .map(|i| {
                            if let Some(v) = matched_trains.get(&(i as i32)) {
                                *v
                            } else {
                                usize::MAX
                            }
                        })
                        .collect();

                    (matched_features, matched_indexes)
                }
            };

            self.prev_computed = Some((train_descriptors, train_keypoints, matched_indexes));

            matched_features
        } else {
            let matched_features = {
                let mut matched_features = vec![MatchedFeature::default(); train_keypoints.len()];
                for i in matched_features.iter_mut().zip(train_keypoints.iter()) {
                    let kp = i.1;
                    let mf = i.0;

                    mf.position = get_vp(kp.pt.x, kp.pt.y);
                }

                matched_features
            };

            let matched_indexes = (0..train_keypoints.len()).into_iter().collect();

            self.prev_computed = Some((train_descriptors, train_keypoints, matched_indexes));

            matched_features
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
