pub use std::io::{Error, ErrorKind, Result};

mod estimator;
mod feature_extractor;
mod matcher;
mod tracker;
mod utils;

pub use estimator::*;
pub use feature_extractor::*;
pub use matcher::*;
pub use tracker::*;
pub use utils::*;

#[cfg(test)]
mod test {
    use std::fs::File;
    use std::io::{BufRead, BufReader};
    use std::path::Path;

    use opencv::{core::*, features2d::*, highgui::*, imgcodecs::*, imgproc::*};

    use super::*;

    #[test]
    fn test() {
        let mut feature_extractor = FeatureExtractor::new();
        let mut matcher = Matcher::new();
    }
}
