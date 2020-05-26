use async_std::prelude::*;
use async_std::sync::{channel, Receiver, Sender};
use async_std::task::spawn;
use nalgebra::*;
use opencv::{core::*, highgui::*, imgproc::*};

use crate::*;

pub struct TrackedViewer {
    tx: Sender<(Mat, Option<i32>)>,
    r_rx: Receiver<Result<()>>,
}

impl TrackedViewer {
    pub fn new() -> Self {
        let (tx, rx) = channel(1);
        let (r_tx, r_rx) = channel(1);

        spawn(async move {
            'a: loop {
                if let Some((img, wait_key_delay)) = rx.recv().await {
                    r_tx.send(
                        async move {
                            imshow("tracked-viewer", &img)
                                .map_err(|_| Error::from(ErrorKind::Other))
                                .and_then(|_| {
                                    if let Some(delay) = wait_key_delay {
                                        wait_key(delay)
                                            .map(|_| ())
                                            .map_err(|_| Error::from(ErrorKind::Other))
                                    } else {
                                        Ok(())
                                    }
                                })
                        }
                        .await,
                    )
                    .await;
                } else {
                    break 'a;
                }
            }
        });

        Self { tx, r_rx }
    }

    pub async fn show_tracked(
        &self,
        src: &Mat,
        tracked: &Tracked,
        wait_key_delay: Option<i32>,
    ) -> Result<()> {
        let mut dst = src.clone().unwrap();

        let img_xo = dst.cols() / 2;
        let img_yo = dst.rows() / 2;
        let translate_vp = |vp: &Vector2<f64>| {
            opencv::core::Point::new(vp.x as i32 + img_xo, -(vp.y as i32) + img_yo)
        };

        for i in 0..tracked.points_count() {
            let mut prev: Option<TrackedPoint> = None;
            'a: for j in 0..tracked.frames_count() {
                if let Some(cur_point) = tracked.get_point(j, i) {
                    if let Some(prev_point) = prev.take() {
                        line(
                            &mut dst,
                            translate_vp(&prev_point.vp_position),
                            translate_vp(&cur_point.vp_position),
                            opencv::core::Scalar::all(256.0),
                            1,
                            LINE_AA,
                            0,
                        )
                        .unwrap();
                    } else {
                        circle(
                            &mut dst,
                            translate_vp(&cur_point.vp_position),
                            10,
                            opencv::core::Scalar::all(256.0),
                            1,
                            LINE_AA,
                            0,
                        )
                        .unwrap();
                    }

                    prev = Some(cur_point);
                } else {
                    break 'a;
                }
            }
        }

        self.tx.send((dst, wait_key_delay)).await;
        self.r_rx.recv().await.unwrap()
    }
}

#[cfg(test)]
mod test {
    #[async_std::test]
    async fn test() {}
}