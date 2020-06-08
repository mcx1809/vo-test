use std::collections::LinkedList;
use std::time::SystemTime;

use nalgebra::*;

use crate::*;

pub struct Tracker {
    max_frames_buffered: u32,
    frames: LinkedList<Frame>,
}

struct Frame {
    timestamp: SystemTime,
    points: Vec<Point>,
}

struct Point {
    prev_index: u32,
    vp_position: Vector2<f64>,
    match_degree: f64,
}

pub struct Tracked {
    frames: Vec<TrackedFrame>,
}

struct TrackedFrame {
    timestamp: SystemTime,
    points: Vec<TrackedPoint>,
}

#[derive(Copy, Clone)]
pub struct TrackedPoint {
    pub vp_position: Vector2<f64>,
}

impl Tracker {
    pub fn new(max_frames_buffered: u32) -> Self {
        Self {
            max_frames_buffered,
            frames: LinkedList::new(),
        }
    }

    pub fn update_matched(
        &mut self,
        timestamp: &SystemTime,
        matched_features: &[feature::MatchedFeature],
    ) {
        let points = matched_features
            .iter()
            .map(|mp| Point {
                prev_index: mp.prev_index,
                vp_position: mp.position,
                match_degree: mp.match_degree,
            })
            .collect();

        let frame = Frame {
            timestamp: *timestamp,
            points,
        };

        if self.frames.len() == self.max_frames_buffered as usize {
            self.frames.pop_back();
        }
        self.frames.push_front(frame);
    }

    pub fn get_tracked(&self) -> Tracked {
        let get_index = |p: &Point| {
            if p.match_degree > 0.0 {
                p.prev_index
            } else {
                u32::MAX
            }
        };

        let mut prev_indexes = Vec::new();
        let mut tracked = Tracked { frames: Vec::new() };
        let mut first_frame = true;
        'a: for matched_frame in &self.frames {
            let tracked_points = if first_frame {
                first_frame = false;

                prev_indexes = matched_frame.points.iter().map(|p| get_index(p)).collect();

                matched_frame
                    .points
                    .iter()
                    .map(|p| TrackedPoint {
                        vp_position: p.vp_position,
                    })
                    .collect()
            } else {
                let mut tracked_points = vec![TrackedPoint::default(); prev_indexes.len()];
                for (prev_index, tp) in prev_indexes.iter_mut().zip(tracked_points.iter_mut()) {
                    if let Some(p) = matched_frame.points.get(*prev_index as usize) {
                        *prev_index = get_index(p);
                        tp.vp_position = p.vp_position;
                    }
                }

                tracked_points
            };

            tracked.frames.push(TrackedFrame {
                timestamp: matched_frame.timestamp,
                points: tracked_points,
            });

            if !prev_indexes.iter().fold(
                false,
                |flag, index| if *index != u32::MAX { true } else { flag },
            ) {
                break 'a;
            }
        }

        tracked
    }
}

impl Tracked {
    pub fn frames_count(&self) -> u32 {
        self.frames.len() as u32
    }

    pub fn points_count(&self) -> u32 {
        if let Some(frame) = self.frames.get(0) {
            frame.points.len() as u32
        } else {
            0
        }
    }

    pub fn get_timestamp(&self, frame_index: u32) -> Option<SystemTime> {
        if let Some(frame) = self.frames.get(frame_index as usize) {
            Some(frame.timestamp)
        } else {
            None
        }
    }

    pub fn get_point(&self, frame_index: u32, point_index: u32) -> Option<TrackedPoint> {
        if let Some(frame) = self.frames.get(frame_index as usize) {
            if let Some(point) = frame.points.get(point_index as usize) {
                if !point.vp_position.x.is_nan() {
                    return Some(*point);
                }
            }
        }

        None
    }
}

impl TrackedPoint {
    fn default() -> Self {
        Self {
            vp_position: Vector2::<f64>::new(f64::NAN, f64::NAN),
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {}
}
