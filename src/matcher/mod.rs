use std::time::SystemTime;

use opencv::core::Mat;

use crate::*;

pub struct Matcher {}

impl Matcher {
    pub fn new() -> Self {
        Self {}
    }

    pub fn update(&mut self, timestamp: &SystemTime, features: &Mat) {}

    pub fn get_matched(&self) -> Vec<MatchedFrame> {
        vec![]
    }
}

pub struct MatchedPoint {}

pub struct MatchedPoints {}

pub struct MatchedFrame {}
