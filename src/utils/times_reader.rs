use std::path::*;
use std::time::{Duration, SystemTime};

use async_std::fs::File;
use async_std::io::BufReader;
use async_std::prelude::*;

use crate::*;

pub struct TimesReader {
    reader: BufReader<File>,
}

impl TimesReader {
    pub async fn open<P: AsRef<Path>>(path: P) -> Result<Self> {
        match File::open(path.as_ref()).await {
            Ok(file) => Ok(Self {
                reader: BufReader::new(file),
            }),
            Err(err) => Err(err),
        }
    }

    pub async fn read_next(&mut self) -> Result<SystemTime> {
        let mut line = String::new();
        match self.reader.read_line(&mut line).await {
            Ok(_) => match line.trim().parse::<f64>() {
                Ok(time) => Ok(SystemTime::UNIX_EPOCH + Duration::from_secs_f64(time)),
                Err(err) => Err(Error::from(ErrorKind::InvalidData)),
            },
            Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[async_std::test]
    async fn test() {
        let mut reader = TimesReader::open("data/00/times.txt").await.unwrap();
        'a: loop {
            match reader.read_next().await {
                Ok(timestamp) => {
                    println!(
                        "{}",
                        timestamp
                            .duration_since(SystemTime::UNIX_EPOCH)
                            .unwrap()
                            .as_secs_f64()
                    );
                }
                Err(err) => {
                    println!("{}", err);
                    break 'a;
                }
            }
        }
    }
}
