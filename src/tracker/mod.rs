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
        let points = features
            .iter()
            .map(|m| FramePoint {
                prev_index: m.prev_index,
                vp_position: m.position,
                match_degree: m.match_degree,
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

        let mut prev_indexes = Vec::new();
        let mut tracked = Tracked { frames: Vec::new() };
        let mut first_frame = true;
        'a: for matched_frame in &self.frames {
            let tracked_frame_points = if first_frame {
                first_frame = false;

                prev_indexes.reserve(matched_frame.points.len());
                let tracked_frame_points = matched_frame
                    .points
                    .iter()
                    .map(|matched_point| {
                        prev_indexes.push(get_index(matched_point));

                        TrackedPoint {
                            vp_position: matched_point.vp_position,
                        }
                    })
                    .collect();

                tracked_frame_points
            } else {
                let mut tracked_frame_points = vec![TrackedPoint::default(); prev_indexes.len()];
                for (prev_index, tracked_point) in
                    prev_indexes.iter_mut().zip(tracked_frame_points.iter_mut())
                {
                    if let Some(matched_point) = matched_frame.points.get(*prev_index as usize) {
                        *prev_index = get_index(matched_point);

                        tracked_point.vp_position = matched_point.vp_position;
                    } else {
                        tracked_point.vp_position.x = f64::NAN;
                        tracked_point.vp_position.y = f64::NAN;
                    }
                }

                tracked_frame_points
            };

            tracked.frames.push(TrackedFrame {
                timestamp: matched_frame.timestamp,
                points: tracked_frame_points,
            });

            if !has_tracked(&prev_indexes) {
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
            vp_position: Vector2::<f64>::new(0.0, 0.0),
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test() {}
}
