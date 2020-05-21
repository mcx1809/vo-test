use std::time::SystemTime;

use crate::*;

pub struct Tracker {}

impl Tracker {
    pub fn new(buffered_frame_num: u32) -> Self {
        Self {}
    }

    pub fn update_features(&mut self, timestamp: &SystemTime, features: &[MatchedFeature]) {}

    pub fn get_tracked(&self, timestamp: &SystemTime, max_frame_num: u32) -> Vec<TrackedPoint> {
        vec![]
    }

    pub fn update_tracked(&mut self, timestamp: &SystemTime, tracked: &[TrackedPoint]) {}

    pub fn delete_tracked(&mut self, timestamp: &SystemTime, tracked: &[TrackedPoint]) {}
}

pub struct TrackedPoint {}
