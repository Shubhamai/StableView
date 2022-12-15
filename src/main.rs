mod camera;
mod filter;
mod model;
mod network;
mod pose;
mod tddfa;
mod utils;

use camera::ThreadedCamera;
use network::SocketNetwork;
use opencv::prelude::{MatTraitConst, MatTraitConstManual};
use std::time::Instant;
use tddfa::TDDFA;

use crate::filter::EuroDataFilter;
use crate::utils::{calc_pose, gen_point2d};

use std::{sync::mpsc, thread, time::Duration};

use tracing::{event, span, Level};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fps = 60;

    let file_appender = tracing_appender::rolling::never("logs", "prefix.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_max_level(Level::INFO)
        .init(); // ! Need to have only 1 log file which resets daily

    tracing::info!("Version {}", env!("CARGO_PKG_VERSION"));
    tracing::info!("fps set to {fps}");

    let mut euro_filter = EuroDataFilter::new();

    // Create a channel to communicate between threads
    let (tx, rx) = mpsc::channel();

    let mut socket_network = SocketNetwork::new(4242);

    let mut thr_cam = ThreadedCamera::setup_camera();
    thr_cam.new_camera_thread(tx);

    // Spawn a thread to run tddfa.run and tddfa.recon_vers on the frames received from the other thread
    let tddfa_runner = thread::spawn(move || {
        let tddfa = TDDFA::new("./assets/data.json", "./assets/mb05_120x120.onnx", 120).unwrap();
        let face_box = [150., 150., 400., 400.];

        let mut frame = rx.recv().unwrap();

        let (param, roi_box) = tddfa
            .run(
                &frame,
                face_box,
                vec![vec![1., 2., 3.], vec![4., 5., 6.], vec![7., 8., 9.]],
                "box",
            )
            .unwrap();
        let pts_3d = tddfa.recon_vers(param, roi_box);

        let (param, roi_box) = tddfa.run(&frame, face_box, pts_3d, "landmark").unwrap();
        let mut pts_3d = tddfa.recon_vers(param, roi_box);

        loop {
            let start_time = Instant::now();

            frame = match rx.try_recv() {
                Ok(result) => result,
                Err(_) => frame.clone(),
            };

            // If frame is valid
            if frame.size().unwrap().width > 0 {
                let (param, roi_box) = tddfa
                    .run(&frame, face_box, pts_3d.clone(), "landmark")
                    .unwrap();
                if (roi_box[2] - roi_box[0]).abs() * (roi_box[3] - roi_box[1]).abs() < 2020. {
                    let (param, roi_box) =
                        tddfa.run(&frame, face_box, pts_3d.clone(), "box").unwrap();
                }
                pts_3d = tddfa.recon_vers(param, roi_box); // ? Commenting this code still seems to output the pose perfectly

                let (p, pose) = calc_pose(&param);

                // println!("{:?}", pts_3d[0][28..48].to_vec());
                let (point2d, mut distance) = gen_point2d(
                    &p,
                    vec![
                        pts_3d[0][28..48].to_vec(),
                        pts_3d[1][28..48].to_vec(),
                        pts_3d[2][28..48].to_vec(),
                    ],
                );
                distance += (pose[0] * 0.2).abs();
                // println!("{}", distance);
                // println!("{:?}", point2d);
                let x = [point2d[0][0], point2d[1][0], point2d[2][0], point2d[3][0]];
                let y = [point2d[0][1], point2d[1][1], point2d[2][1], point2d[3][1]];
                // println!("{:?}, {:?}", x);
                let mut centroid = [
                    x.iter().sum::<f32>() / (x.len()) as f32,
                    y.iter().sum::<f32>() / (y.len()) as f32,
                ];
                // * disbling the multiplying pose with distance (pose[0]*(distance/31), pose[1]*(distance/27)), it seems to causing jitting even when blinking eyes or smiling
                centroid[0] += pose[0]; // * When very close to the camera, the head pose invariant seems to does't work, to miltgate the issue, we use this
                centroid[1] -= pose[1] * 1.7; // * 31 & 27 represent the distance where head pose invariant is fully solved, and we use this ratio to make it work for closer distance
                if pose[2] > 0. {
                    centroid[0] += pose[2].abs()
                } else {
                    centroid[0] -= pose[2].abs()
                }

                let mut data = [
                    centroid[0] as f64,
                    -centroid[1] as f64,
                    // 0.,
                    // 0.,
                    f64::from(distance),
                    // 0.,
                    f64::from(pose[0]),
                    f64::from(-pose[1]),
                    f64::from(pose[2]),
                ];

                data = euro_filter.filter_data(data);

                socket_network.send(data);

                let elapsed_time = start_time.elapsed();
                thread::sleep(Duration::from_millis(
                    (((1000 / fps) - elapsed_time.as_millis()).max(0))
                        .try_into()
                        .unwrap(),
                ));
                // println!(
                //     "{}x{} @ {} ms | {:.1} {:.1}",
                //     frame.cols(),
                //     frame.rows(),
                //     elapsed_time.as_millis(),
                //     pose[0],
                //     pose[1]
                // );
            }
        }
    });

    // Wait for the threads to finish
    // thr_cam.stop();
    tddfa_runner.join().unwrap();

    Ok(())
}
