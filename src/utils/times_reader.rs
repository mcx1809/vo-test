use std::path::*;
use std::time::SystemTime;

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
                .map(|time| seconds_to_timestamp(time))
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
                    println!("timestamp: {}", timestamp_to_seconds(&timestamp));
                }
                Err(err) => {
                    println!("{}", err);
                    break 'a;
                }
            }
        }
    }
}
