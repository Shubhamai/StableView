/// Running camera on a seperate thread and returning the frames
use opencv::{
    prelude::{Mat, VideoCaptureTrait, VideoCaptureTraitConst},
    videoio,
};
use std::{
    sync::{
        self,
        atomic::{AtomicBool, Ordering},
        mpsc::Sender,
    },
    thread,
};

use std::collections::HashMap;

pub struct ThreadedCamera {
    cam_thread: Option<thread::JoinHandle<()>>, // Storing the thread
    keep_running: sync::Arc<AtomicBool>,        // Signal to stop the thread
}

impl ThreadedCamera {
    pub fn get_available_cameras() -> Result<HashMap<String, i32>, nokhwa::NokhwaError> {
        //Result<Vec<CameraInfo>, nokhwa::NokhwaError> {
        // let mut devices_list: Vec<CameraInfo> = vec![];
        let mut devices_list = HashMap::new();

        let available_devices = nokhwa::query(nokhwa::native_api_backend().unwrap());

        match available_devices {
            Ok(available_devices) => {
                for device_info in available_devices {
                    tracing::info!(
                        "Detected : {} @ index {}",
                        device_info.human_name(),
                        device_info.index()
                    );
                    devices_list.insert(device_info.human_name(), device_info.index().as_index().unwrap() as i32);
                }
            }
            Err(error) => {
                tracing::error!(
                    "Unable to read camera devices : {:?}. Setting default (Deault Device, 0)",
                    error
                );
                devices_list.insert("Deault Device".to_string(), 0);
            }
        };

        Ok(devices_list)
    }

    pub fn start_camera_thread(tx: Sender<Mat>, camera_index: i32) -> Self {
        // Serving as a signal to stop the thread when needed
        let keep_running = sync::Arc::new(AtomicBool::new(false));
        keep_running.store(true, Ordering::SeqCst);

        let cloned_keep_running = keep_running.clone();

        let mut cam = videoio::VideoCapture::new(camera_index, videoio::CAP_ANY)
            .expect("Unable to setup the camera with index {camera_index}");
        let opened = videoio::VideoCapture::is_opened(&cam)
            .expect("Unable to open the camera with index {camera_index}");

        assert!(opened, "Unable to open default camera!");

        let cam_thread = Some(thread::spawn(move || {
            // Running loop as long as keep_running is true
            while cloned_keep_running.load(Ordering::SeqCst) {
                // Reading frame
                let mut frame = Mat::default();
                cam.read(&mut frame).unwrap();

                // Send the frame to the other thread for processing
                if tx.send(frame).is_err() {
                    break;
                }
            }
        }));

        Self {
            cam_thread,
            keep_running,
        }
    }

    pub fn shutdown(&mut self) {
        println!("Shutting down camera thread...");

        self.keep_running.store(false, Ordering::SeqCst);
        self.cam_thread
            .take()
            .expect("Called stop on non-running thread")
            .join()
            .expect("Could not join spawned thread");
    }
}

#[test]
#[ignore = "Can only test this offline since it requires webcam, run cargo test -- --ignored"]
pub fn test_threaded_camera() {
    use sync::mpsc;

    let (tx, rx) = mpsc::channel();

    println!("{:?}", ThreadedCamera::get_available_cameras());

    let mut thr_cam = ThreadedCamera::start_camera_thread(tx, 0);

    for _ in 0..100 {
        let _frame = rx.recv().unwrap();
    }

    thr_cam.shutdown();
}
