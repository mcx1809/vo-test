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
        File::open(path.as_ref()).await.map(|file| Self {
            reader: BufReader::new(file),
        })
    }

    pub async fn read_next(&mut self) -> Result<SystemTime> {
        let mut line = String::new();
        self.reader.read_line(&mut line).await.and_then(|_| {
            line.trim()
                .parse::<f64>()
                .map(|time| SystemTime::UNIX_EPOCH + Duration::from_secs_f64(time))
                .map_err(|_| Error::from(ErrorKind::InvalidData))
        })
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
                        "timestamp: {}",
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
