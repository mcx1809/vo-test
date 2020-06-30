use std::path::*;
use std::time::SystemTime;

use async_std::fs::File;
use async_std::io::BufReader;
use async_std::prelude::*;
use opencv::core::*;
use opencv::imgcodecs::*;

use crate::*;

pub struct ImagesReader {
    images_dir: PathBuf,
    index_file_reader: BufReader<File>,
    read_ahead: Option<Result<(SystemTime, Vec<u8>)>>,
}

impl ImagesReader {
    pub async fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        match File::open(path.as_ref().join("rgb.txt")).await {
            Ok(file) => {
                let mut s = Self {
                    images_dir: path.as_ref().to_path_buf(),
                    index_file_reader: BufReader::new(file),
                    read_ahead: None,
                };

                s.read_ahead = Some(s.read_next_file().await);

                Ok(s)
            }
            Err(err) => Err(err),
        }
    }

    pub async fn read_next(&mut self) -> Result<(SystemTime, Mat)> {
        match self.read_ahead.take().unwrap() {
            Ok((time, image_buf)) => {
                let (ra, ri) = self
                    .read_next_file()
                    .join(Self::decode_file(image_buf))
                    .await;

                self.read_ahead = Some(ra);
                if let Ok(image) = ri {
                    Ok((time, image))
                } else {
                    Err(Error::from(ErrorKind::InvalidData))
                }
            }
            Err(err) => Err(err),
        }
    }

    async fn read_next_file(&mut self) -> Result<(SystemTime, Vec<u8>)> {
        loop {
            let mut line = String::new();
            match self.index_file_reader.read_line(&mut line).await {
                Ok(len) => {
                    if len > 0 {
                        if line.chars().next().unwrap() != '#' {
                            let mut sp = line.trim().split_ascii_whitespace();
                            if let Some(f) = sp.next() {
                                if let Ok(time) = f.parse::<f64>() {
                                    if let Some(f) = sp.next() {
                                        match File::open(self.images_dir.join(f)).await {
                                            Ok(mut file) => {
                                                let mut buf = vec![];
                                                match file.read_to_end(&mut buf).await {
                                                    Ok(_) => {
                                                        return Ok((
                                                            seconds_to_timestamp(time),
                                                            buf,
                                                        ));
                                                    }
                                                    Err(err) => {
                                                        return Err(err);
                                                    }
                                                }
                                            }
                                            Err(err) => {
                                                return Err(err);
                                            }
                                        }
                                    }
                                }
                            }

                            return Err(Error::from(ErrorKind::InvalidData));
                        }
                    } else {
                        return Err(Error::from(ErrorKind::NotFound));
                    }
                }
                Err(err) => {
                    return Err(err);
                }
            }
        }
    }

    async fn decode_file(buf: Vec<u8>) -> Result<Mat> {
        // TODO: 额外的拷贝
        if let Ok(image) = imdecode(&Vector::<u8>::from_iter(buf.into_iter()), IMREAD_GRAYSCALE) {
            Ok(image)
        } else {
            Err(Error::from(ErrorKind::InvalidData))
        }
    }
}

#[cfg(test)]
mod test {
    use opencv::highgui::*;

    use super::*;

    #[async_std::test]
    async fn test() {
        let mut reader = ImagesReader::open("data/rgbd_dataset_freiburg1_xyz")
            .await
            .unwrap();

        'a: loop {
            match reader.read_next().await {
                Ok((time, img)) => {
                    println!("{}", timestamp_to_seconds(&time));
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
