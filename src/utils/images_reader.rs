use std::path::*;

use async_std::fs::File;
use async_std::prelude::*;
use opencv::core::*;
use opencv::imgcodecs::*;

use crate::*;

pub struct ImagesReader {
    dir: PathBuf,
    current_index: u32,
}

impl ImagesReader {
    pub fn new<P: AsRef<Path>>(dir: P) -> Self {
        Self {
            dir: dir.as_ref().to_path_buf(),
            current_index: 0,
        }
    }

    // TODO: 并行优化
    pub async fn read_next(&mut self) -> Result<Mat> {
        let file_name = format!("{:06}.png", self.current_index);
        println!("{}", file_name);

        match File::open(self.dir.join(file_name)).await {
            Ok(mut file) => {
                let mut buf = vec![];
                match file.read_to_end(&mut buf).await {
                    Ok(_) => {
                        match imdecode(&Vector::<u8>::from_iter(buf.into_iter()), IMREAD_GRAYSCALE)
                        {
                            Ok(img) => {
                                self.current_index += 1;
                                Ok(img)
                            }
                            Err(_) => Err(Error::from(ErrorKind::InvalidData)),
                        }
                    }
                    Err(err) => Err(err),
                }
            }
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod test {
    use opencv::highgui::*;

    use super::*;

    #[async_std::test]
    async fn test() {
        let mut reader = ImagesReader::new("data/00/image_0");

        'a: loop {
            match reader.read_next().await {
                Ok(img) => {
                    imshow("vo-test", &img).unwrap();
                    wait_key(20).unwrap();
                }
                Err(err) => {
                    println!("{}", err);
                    break 'a;
                }
            }
        }
    }
}
