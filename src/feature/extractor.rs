use opencv::{core::*, features2d::*};

pub struct Extractor {
    orb: Ptr<dyn ORB>,
}

impl Extractor {
    pub fn new() -> Self {
        Self {
            orb: ORB::create(500, 2.0, 8, 31, 0, 2, ORB_ScoreType::FAST_SCORE, 31, 100).unwrap(),
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
            keypoints,
            descriptors,
        }
    }
}

pub struct Features {
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

        //let ppi = SVD::new(p, true, true).pseudo_inverse(1e-6).unwrap();
        //println!("{} {} {} {}", p, ppi, p * ppi, ppi * p);

        let x = Vector4::new(-2.0, 0.0, 15.0, 1.0);
        let y = p * x;

        println!("{} {}", y[0] / y[2], y[1] / y[2]);
        //println!("{} {} {}", y[0], y[1], y[2]);

        //println!("{}", ppi * y);
        //let y1 = Vector3::<f64>::new(y[0] / y[2], y[1] / y[2], 1.0);
        //println!("{}", ppi * y1);
    }
}
