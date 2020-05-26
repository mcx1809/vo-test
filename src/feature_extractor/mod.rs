use opencv::{core::*, features2d::*};

pub struct FeatureExtractor {
    orb: Ptr<dyn ORB>,
}

impl FeatureExtractor {
    pub fn new() -> Self {
        Self {
            orb: ORB::create(500, 2.0, 8, 31, 0, 2, ORB_FAST_SCORE, 31, 200).unwrap(),
        }
    }

    pub async fn get_features(&mut self, src: &Mat) -> Features {
        let mut keypoints = Vector::<KeyPoint>::new();
        let mut descriptors = Mat::default().unwrap();
        self.orb
            .detect_and_compute(
                src,
                &no_array().unwrap(),
                &mut keypoints,
                &mut descriptors,
                false,
            )
            .unwrap();

        Features {
            img_rows: src.rows(),
            img_cols: src.cols(),
            keypoints,
            descriptors,
        }
    }
}

pub struct Features {
    pub img_rows: i32,
    pub img_cols: i32,
    pub keypoints: Vector<KeyPoint>,
    pub descriptors: Mat,
}

#[cfg(test)]
mod test {
    use nalgebra::*;

    #[test]
    fn test() {
        let p = Matrix3x4::new(
            7.188560000000e+02,
            0.000000000000e+00,
            6.071928000000e+02,
            0.000000000000e+00,
            0.000000000000e+00,
            7.188560000000e+02,
            1.852157000000e+02,
            0.000000000000e+00,
            0.000000000000e+00,
            0.000000000000e+00,
            1.000000000000e+00,
            0.000000000000e+00,
        );

        let x = Vector4::new(0.0, 0.0, 10.0, 1.0);
        let y = p * x;

        println!("{} {}", y[0] / y[2], y[1] / y[2]);
    }
}
