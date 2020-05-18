use opencv::core::Mat;

pub struct FeatureExtractor {}

impl FeatureExtractor {
    pub fn new() -> Self {
        Self {}
    }

    pub fn get_features(&mut self, src: &Mat) -> Mat {
        Mat::default().unwrap()
    }
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
