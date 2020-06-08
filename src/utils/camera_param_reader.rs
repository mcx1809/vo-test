use std::path::*;

use async_std::fs::File;
use async_std::io::BufReader;
use async_std::prelude::*;

use nalgebra::*;

use crate::*;

pub async fn read_camera_param<P: AsRef<Path>>(path: P) -> Result<Vec<Matrix3x4<f64>>> {
    match File::open(path.as_ref()).await {
        Ok(file) => {
            let mut reader = BufReader::new(file);

            // TODO: 读取行数不受限
            let mut params = Vec::new();
            'a: loop {
                let mut line = String::new();
                // TODO: 读取长度不受限
                match reader.read_line(&mut line).await {
                    Ok(len) => {
                        if len != 0 {
                            let mut vv = [0.0; 12];
                            match line.split_ascii_whitespace().try_fold(0, |i, field| {
                                let r = if i >= 1 {
                                    field
                                        .parse::<f64>()
                                        .map(|v| {
                                            vv[i - 1] = v;
                                            i + 1
                                        })
                                        .map_err(|_| Error::from(ErrorKind::InvalidData))
                                } else {
                                    Ok(i + 1)
                                };

                                if i == 12 {
                                    params.push(Matrix3x4::from_row_slice(&vv));
                                    Err(Error::from(ErrorKind::Other))
                                } else {
                                    r
                                }
                            }) {
                                Ok(_) => {
                                    return Err(Error::from(ErrorKind::InvalidData));
                                }
                                Err(err) => {
                                    if err.kind() != ErrorKind::Other {
                                        return Err(err);
                                    }
                                }
                            }
                        } else {
                            break 'a;
                        }
                    }
                    Err(err) => {
                        return Err(err);
                    }
                }
            }

            Ok(params)
        }
        Err(err) => Err(err),
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[async_std::test]
    async fn test() {
        for p in read_camera_param("data/00/calib.txt").await.unwrap() {
            println!("{}", p);
        }
    }
}
