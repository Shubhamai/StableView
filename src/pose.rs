use crate::tddfa::TDDFA;

use crate::{utils::calc_pose};
use opencv::prelude::Mat;
use std::{
    sync::{
        self,
        atomic::{AtomicBool, Ordering},
        mpsc::{Receiver},
    },
    thread,
};

pub struct ProcessHeadPose {
    // tddfa: TDDFA,
    // face_box: [f32; 4],
    // pts_3d: Vec<Vec<f32>>,
    tddfa_thread: Option<thread::JoinHandle<()>>,
    alive: sync::Arc<AtomicBool>,
}

impl ProcessHeadPose {
    pub fn new(_data_fp:String, _model_fp:String, _image_size:u8) -> Self {
        Self {
            tddfa_thread: None,
            alive: sync::Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start_tddfa_thread(&mut self, rx: Receiver<Mat>) {
        self.alive.store(true, Ordering::SeqCst);
        let alive = self.alive.clone();

        self.tddfa_thread = Some(thread::spawn(move || {
            
            let tddfa =
                TDDFA::new("./assets/data.json", "./assets/mb05_120x120.onnx", 120).unwrap();
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

            while alive.load(Ordering::SeqCst) {
                frame = match rx.try_recv() {
                    Ok(result) => result,
                    Err(_) => frame.clone(),
                };

                let (param, roi_box) = tddfa
                    .run(&frame, face_box, pts_3d.clone(), "landmark")
                    .unwrap();
                pts_3d = tddfa.recon_vers(param, roi_box); // ? Commenting this code still seems to output the pose perfectly

                let (_p, _pose) = calc_pose(&param);

                
            }
        }));
    }

    pub fn stop(&mut self) {
        self.alive.store(false, Ordering::SeqCst);
        self.tddfa_thread
            .take()
            .expect("Called stop on non-running thread")
            .join()
            .expect("Could not join spawned thread");
    }
}

#[test]
pub fn test_process_head_pose() {
    // let (tx, _rx) = mpsc::channel();

    // let mut thr_cam = ThreadedCamera::setup_camera();
    // thr_cam.new_camera_thread(tx);

    // let mut head_pose = ProcessHeadPose::new();
    // head_pose.start_tddfa_thread(rx);
    // head_pose.stop();
}
