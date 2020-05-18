use std::fs::File;
use std::io::{BufRead, BufReader};
use std::path::Path;

use opencv::{core::*, features2d::*, highgui::*, imgcodecs::*, imgproc::*};

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
                                            imshow("vo-test", &draw_features_2d(&img)).unwrap();
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
