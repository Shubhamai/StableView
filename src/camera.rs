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

pub struct ThreadedCamera {
    cam: nokhwa::CameraInfo,
    cam_thread: Option<thread::JoinHandle<()>>,
    alive: sync::Arc<AtomicBool>,
}

impl ThreadedCamera {
    pub fn setup_camera() -> Self {
        let dev = nokhwa::query_devices(nokhwa::CaptureAPIBackend::Auto).unwrap();

        for device_info in &dev {
            tracing::info!(
                "Detected : {} @ index {}",
                device_info.human_name(),
                device_info.index()
            );
        }

        Self {
            cam: dev[0].clone(),
            cam_thread: None,
            alive: sync::Arc::new(AtomicBool::new(false)),
        }
    }

    pub fn start_camera_thread(&mut self, tx: Sender<Mat>) {
        self.alive.store(true, Ordering::SeqCst);

        let alive = self.alive.clone();

        tracing::info!(
            "Using : {} @ index {}",
            self.cam.human_name(),
            self.cam.index()
        );

        let mut cam =
            videoio::VideoCapture::new(self.cam.index() as i32, videoio::CAP_ANY).unwrap(); // videoio::CAP_ANY, CAP_V4L2, // 0 is the default camera
        let opened = videoio::VideoCapture::is_opened(&cam).unwrap();

        assert!(opened, "Unable to open default camera!");

        self.cam_thread = Some(thread::spawn(move || {
            while alive.load(Ordering::SeqCst) {
                // Reading frame
                let mut frame = Mat::default();
                cam.read(&mut frame).unwrap();

                // Send the frame to the other thread for processing
                if tx.send(frame).is_err() {
                    break;
                }
            }
        }));
    }

    // pub fn get_frame(&self, rx:Receiver<Mat>){
    //     let frame = match rx.try_recv() {
    //         Ok(result) => result,
    //         Err(_) => frame.clone()
    //     };
    // }

    pub fn shutdown(&mut self) {
        println!("Shutting down camera thread...");

        self.alive.store(false, Ordering::SeqCst);
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

    let mut thr_cam = ThreadedCamera::setup_camera();
    thr_cam.start_camera_thread(tx);

    for _ in 0..100 {
        let _frame = rx.recv().unwrap();
    }

    thr_cam.shutdown();
}
