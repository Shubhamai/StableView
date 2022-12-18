///
use crate::filter::EuroDataFilter;
use crate::network::SocketNetwork;
use crate::tddfa::Tddfa;
use crate::utils::{calc_pose, gen_point2d};

use opencv::prelude::Mat;
use std::{
    io,
    sync::mpsc::{self, Receiver, TryRecvError},
    thread,
    time::{Duration, Instant},
};

pub struct ProcessHeadPose {
    tddfa: Tddfa,
    pts_3d: Vec<Vec<f32>>,
    user_input_thread: Option<thread::JoinHandle<()>>,
    face_box: [f32; 4],
    fps: u128,
}

impl ProcessHeadPose {
    pub fn new(data_fp: &str, landmark_model_fp: &str, image_size: i32, fps: u128) -> Self {
        let tddfa = Tddfa::new(data_fp, landmark_model_fp, image_size).unwrap();

        Self {
            tddfa,
            pts_3d: vec![vec![1., 2., 3.], vec![4., 5., 6.], vec![7., 8., 9.]],
            user_input_thread: None,
            face_box: [150., 150., 400., 400.],
            fps,
        }
    }

    fn get_coordintes_and_depth(
        &self,
        pose: [f32; 3],
        mut distance: f32,
        point2d: Vec<Vec<f32>>,
    ) -> ([f32; 2], f32) {
        distance += (pose[0] * 0.2).abs();

        let x = [point2d[0][0], point2d[1][0], point2d[2][0], point2d[3][0]];
        let y = [point2d[0][1], point2d[1][1], point2d[2][1], point2d[3][1]];

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

        (centroid, distance)
    }

    pub fn single_iter(&mut self, frame: &Mat) -> [f32; 6] {
        let start_time = Instant::now();

        let (mut param, mut roi_box) = self
            .tddfa
            .run(frame, self.face_box, &self.pts_3d, "landmark")
            .unwrap();

        if (roi_box[2] - roi_box[0]).abs() * (roi_box[3] - roi_box[1]).abs() < 2020. {
            (param, roi_box) = self
                .tddfa
                .run(frame, self.face_box, &self.pts_3d, "box")
                .unwrap();
        }

        self.pts_3d = self.tddfa.recon_vers(param, roi_box); // ? Commenting this code still seems to output the pose perfectly

        let (p, pose) = calc_pose(&param);

        let (point2d, distance) = gen_point2d(
            &p,
            vec![
                self.pts_3d[0][28..48].to_vec(),
                self.pts_3d[1][28..48].to_vec(),
                self.pts_3d[2][28..48].to_vec(),
            ],
        );

        let (centroid, distance) = self.get_coordintes_and_depth(pose, distance, point2d);

        let data = [
            centroid[0],
            -centroid[1],
            distance,
            pose[0],
            -pose[1],
            pose[2],
        ];

        let elapsed_time = start_time.elapsed();
        thread::sleep(Duration::from_millis(
            (((1000 / self.fps) - elapsed_time.as_millis()).max(0))
                .try_into()
                .unwrap(),
        ));

        data
    }

    pub fn start_loop(
        &mut self,
        rx: Receiver<Mat>,
        mut filter: EuroDataFilter,
        mut socket: SocketNetwork,
    ) {
        // let stdin_channel = self.spawn_stdin_channel();

        let mut frame = rx.recv().unwrap();
        let mut data;

        loop {
            // match stdin_channel.try_recv() {
            //     Ok(key) => {
            //         if key.trim().is_empty() {
            //             break;
            //         }
            //     }
            //     Err(TryRecvError::Empty) => {}
            //     Err(TryRecvError::Disconnected) => break,
            // }

            frame = match rx.try_recv() {
                Ok(result) => result,
                Err(_) => frame.clone(),
            };

            data = self.single_iter(&frame);

            data = filter.filter_data(data);

            socket.send(data);
        }
    }

    // fn spawn_stdin_channel(&mut self) -> Receiver<String> {
    //     let (tx, rx) = mpsc::channel::<String>();
    //     println!("Press Enter to exit.");
    //     self.user_input_thread = Some(thread::spawn(move || loop {
    //         let mut buffer = String::new();
    //         io::stdin().read_line(&mut buffer).unwrap();
    //         tx.send(buffer).unwrap();
    //     }));
    //     rx
    // }

    // pub fn shutdown(&mut self) {
    //     println!("Shutting down user input thread...");

    //     self.user_input_thread
    //         .take()
    //         .expect("Called stop on non-running thread")
    //         .join()
    //         .unwrap();
    // }
}

#[test]
#[ignore = "Can only test this offline since it requires webcam, run cargo test -- --ignored"]
pub fn test_process_head_pose() {
    use crate::camera::ThreadedCamera;

    let euro_filter = EuroDataFilter::new(0.0025, 0.01);
    let socket_network = SocketNetwork::new((127, 0, 0, 1), 4242);

    let (tx, rx) = mpsc::channel();

    let mut thr_cam = ThreadedCamera::start_camera_thread(tx, 0);
    // thr_camstart_camera_thread(tx);

    let mut head_pose =
        ProcessHeadPose::new("./assets/data.json", "./assets/mb05_120x120.onnx", 120, 60);

    head_pose.start_loop(rx, euro_filter, socket_network);

    // head_pose.shutdown();
    thr_cam.shutdown();
}
