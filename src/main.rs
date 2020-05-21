use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use opencv::{core::*, features2d::*, highgui::*, imgcodecs::*, imgproc::*};

struct Tracker {
    orb: Ptr<dyn ORB>,
    matcher: Ptr<BFMatcher>,
    prev_computed: Option<(Vector<KeyPoint>, Mat)>,
}

impl Tracker {
    fn new() -> Self {
        Self {
            orb: ORB::create(500, 2.0, 8, 31, 0, 2, ORB_FAST_SCORE, 31, 150).unwrap(),
            matcher: BFMatcher::create(NORM_HAMMING, true).unwrap(),
            prev_computed: None,
        }
    }

    fn process(&mut self, frame: &Mat) -> Option<Vec<MatchedPoint>> {
        let mut train_keypoints = Vector::<KeyPoint>::new();
        let mut train_descriptors = Mat::default().unwrap();
        self.orb
            .detect_and_compute(
                frame,
                &no_array().unwrap(),
                &mut train_keypoints,
                &mut train_descriptors,
                false,
            )
            .unwrap();

        if let Some((_, query_descriptors)) = self.prev_computed.take() {
            let mut matches = Vector::<DMatch>::new();
            self.matcher
                .train_match(
                    &query_descriptors,
                    &train_descriptors,
                    &mut matches,
                    &no_array().unwrap(),
                )
                .unwrap();

            let mut matched_points = Vec::<MatchedPoint>::new();
            for m in matches {
                let kp = train_keypoints.get(m.train_idx as usize).unwrap();
                matched_points.push(MatchedPoint {
                    pt: kp.pt,
                    distance: m.distance,
                });
            }

            self.prev_computed = Some((train_keypoints, train_descriptors));

            Some(matched_points)
        } else {
            self.prev_computed = Some((train_keypoints, train_descriptors));

            None
        }
    }
}

struct MatchedPoint {
    pt: Point2f,
    distance: f32,
}

fn draw_matches(src: &Mat, matched_points: &[MatchedPoint]) -> Mat {
    let mut dst = src.clone().unwrap();

    for mp in matched_points {
        circle(
            &mut dst,
            Point::new(mp.pt.x as i32, mp.pt.y as i32),
            10, /* (mp.distance / 15.0) as i32*/
            Scalar::all(256.0),
            1,
            LINE_AA,
            0,
        )
        .unwrap();
    }

    dst
}

fn draw_features_2d(src: &Mat) -> Mat {
    let mut dst =
        Mat::new_rows_cols_with_default(src.rows(), src.cols(), CV_32FC1, Scalar::all(0.0))
            .unwrap();

    corner_harris(&src, &mut dst, 2, 3, 0.04, BORDER_DEFAULT).unwrap();

    let mut dst_norm = Mat::default().unwrap();
    normalize(
        &dst,
        &mut dst_norm,
        0.0,
        255.0,
        NORM_MINMAX,
        CV_32FC1,
        &mut Mat::default().unwrap(),
    )
    .unwrap();

    let mut dst_norm_scaled = Mat::default().unwrap();
    convert_scale_abs(&dst_norm, &mut dst_norm_scaled, 1.0, 0.0).unwrap();

    let mut out = src.clone().unwrap();
    for j in 0..dst_norm.rows() {
        for i in 0..dst_norm.cols() {
            let v = *dst_norm.at_2d::<f32>(j, i).unwrap();
            if v > 125.0 {
                circle(
                    &mut out,
                    Point::new(i, j),
                    (v / 15.0) as i32,
                    Scalar::all(256.0),
                    1,
                    LINE_AA,
                    0,
                )
                .unwrap();
            }
        }
    }

    out
}

fn main() {
    let dataset_dir = Path::new("data").join("00");

    let times_file_path = dataset_dir.join("times.txt");
    match File::open(times_file_path) {
        Ok(times_file) => {
            let mut times_reader = BufReader::new(times_file);

            let img_dir = dataset_dir.join("image_0");
            let mut img_count = 0;

            let mut tracker = Tracker::new();

            'a: loop {
                let mut line = String::new();
                match times_reader.read_line(&mut line) {
                    Ok(count) => {
                        if count > 0 {
                            match line.trim().parse::<f64>() {
                                Ok(time) => {
                                    println!("time: {}", time);

                                    let img_path = img_dir.join(format!("{:06}.png", img_count));
                                    img_count = img_count + 1;

                                    match imread(img_path.to_str().unwrap(), IMREAD_GRAYSCALE) {
                                        Ok(img) => {
                                            if let Some(matched_points) = tracker.process(&img) {
                                                imshow(
                                                    "vo-test",
                                                    &draw_matches(&img, &matched_points),
                                                )
                                                .unwrap();
                                            }

                                            wait_key(0).unwrap();
                                        }
                                        Err(err) => {
                                            println!("{}", err);
                                            break 'a;
                                        }
                                    }
                                }
                                Err(err) => {
                                    println!("{}", err);
                                    break 'a;
                                }
                            }
                        } else {
                            break 'a;
                        }
                    }
                    Err(err) => println!("{}", err),
                }
            }
        }
        Err(err) => println!("{}", err),
    }
}
