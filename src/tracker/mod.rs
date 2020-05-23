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
    points: Vec<FramePoint>,
}

struct FramePoint {
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

    pub fn update_matched(&mut self, timestamp: &SystemTime, features: &[MatchedFeature]) {
        let points = {
            let mut points = Vec::with_capacity(features.len());

            for m in features {
                points.push(FramePoint {
                    prev_index: m.prev_index,
                    vp_position: m.position,
                    match_degree: m.match_degree,
                });
            }

            points
        };

        let frame = Frame {
            timestamp: *timestamp,
            points,
        };

        self.frames.push_front(frame);
        if self.frames.len() > self.max_frames_buffered as usize {
            self.frames.pop_back();
        }
    }

    pub fn get_tracked(&self) -> Tracked {
        let mut prev_indexes = Vec::new();
        let mut tracked = Tracked { frames: Vec::new() };

        let has_tracked = |indexes: &[u32]| {
            indexes.iter().fold(
                false,
                |_, index| if *index != u32::MAX { true } else { false },
            )
        };

        let get_index = |matched_point: &FramePoint| {
            if matched_point.match_degree > 0.0 {
                matched_point.prev_index
            } else {
                u32::MAX
            }
        };

        let mut first_frame = true;
        'a: for matched_frame in &self.frames {
            if first_frame {
                first_frame = false;

                let mut tracked_frame = TrackedFrame {
                    timestamp: matched_frame.timestamp,
                    points: Vec::new(),
                };

                prev_indexes.reserve(matched_frame.points.len());
                tracked_frame.points.reserve(matched_frame.points.len());
                for matched_point in &matched_frame.points {
                    prev_indexes.push(get_index(matched_point));

                    tracked_frame.points.push(TrackedPoint {
                        vp_position: matched_point.vp_position,
                    });
                }

                tracked.frames.push(tracked_frame);
                if !has_tracked(&prev_indexes) {
                    break 'a;
                }
            } else {
                let mut tracked_frame = TrackedFrame {
                    timestamp: matched_frame.timestamp,
                    points: vec![TrackedPoint::default(); prev_indexes.len()],
                };

                for (prev_index, tracked_point) in
                    prev_indexes.iter_mut().zip(tracked_frame.points.iter_mut())
                {
                    if let Some(matched_point) = matched_frame.points.get(*prev_index as usize) {
                        *prev_index = get_index(matched_point);

                        tracked_point.vp_position = matched_point.vp_position;
                    }
                }

                tracked.frames.push(tracked_frame);
                if !has_tracked(&prev_indexes) {
                    break 'a;
                }
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
                return Some(*point);
            }
        }

        None
    }
}

impl TrackedPoint {
    fn default() -> Self {
        Self {
            vp_position: Vector2::<f64>::new(0.0, 0.0),
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {}
}
