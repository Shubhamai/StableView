/// Running camera on a seperate thread and returning the frames
use crossbeam_channel::Sender;

use opencv::{
    prelude::{Mat, VideoCaptureTrait, VideoCaptureTraitConst},
    videoio,
};
use std::{
    sync::{
        self,
        atomic::{AtomicBool, Ordering},
    },
    thread,
};

use std::collections::HashMap;

use crate::structs::camera::ThreadedCamera;

use anyhow::Result;

impl ThreadedCamera {
    pub fn get_available_cameras() -> Result<HashMap<String, i32>> {
        let mut devices_list = HashMap::new();

        let available_devices = nokhwa::query(match nokhwa::native_api_backend() {
            Some(native_api_backend) => native_api_backend,
            None => {
                return Err(anyhow::anyhow!(
                    "Unable to read native API backend for camera."
                ));
            }
        });

        match available_devices {
            Ok(available_devices) => {
                if available_devices.is_empty() {
                    tracing::error!(
                        "No Camera devices found. Setting default (No Device Found, -1)",
                    );
                    devices_list.insert("No Device Found".to_string(), -1);
                } else {
                    for device_info in available_devices {
                        tracing::warn!(
                            "Detected : {} @ index {}",
                            device_info.human_name(),
                            device_info.index()
                        );
                        devices_list.insert(
                            format!("{:<4} {}", device_info.human_name(), device_info.index()),
                            match device_info.index().as_index() {
                                Ok(index) => index as i32,
                                Err(error) => {
                                    tracing::error!("Unable to get camera index : {:?}, adding (No Device Found, -1)", error);
                                    devices_list.insert("No Device Found".to_string(), -1);
                                    return Ok(devices_list);
                                }
                            },
                        );
                    }
                }
            }
            Err(error) => {
                tracing::error!("Unable to read camera devices : {:?}", error);
                return Err(anyhow::anyhow!(
                    "Unable to read camera devices : {:?}",
                    error
                ));
                // devices_list.insert("No Device Found".to_string(), 0);
            }
        };

        Ok(devices_list)
    }

    pub fn start_camera_thread(
        tx: Sender<Mat>,
        camera_index: i32,
        camera_name: String,
    ) -> Result<Self> {
        // Serving as a signal to stop the thread when needed
        let keep_running = sync::Arc::new(AtomicBool::new(false));
        keep_running.store(true, Ordering::SeqCst);

        let cloned_keep_running = keep_running.clone();

        let mut cam = match videoio::VideoCapture::new(camera_index, videoio::CAP_ANY) {
            Ok(cam) => cam,
            Err(error) => {
                return Err(anyhow::anyhow!(
                    "Unable to open camera {camera_name} with index {camera_index} : {:?}",
                    error
                ));
            }
        };
        let opened = match videoio::VideoCapture::is_opened(&cam) {
            Ok(opened) => opened,
            Err(error) => {
                return Err(anyhow::anyhow!(
                    "Unable to open camera {camera_name} with index {camera_index} : {:?}",
                    error
                ));
            }
        };

        if !opened {
            return Err(anyhow::anyhow!("Unable to open the camera!"));
        }

        let cam_thread = Some(thread::spawn(move || {
            // Running loop as long as keep_running is true
            while cloned_keep_running.load(Ordering::SeqCst) {
                // Reading frame
                let mut frame = Mat::default();
                match cam.read(&mut frame) {
                    Ok(_) => (),
                    Err(error) => {
                        panic!("Unable to read frame from camera : {:?}", error);
                        // return Err(anyhow::anyhow!(
                        //     "Unable to read frame from camera : {:?}",
                        //     error
                        // ));
                    }
                }

                // Send the frame to the other thread for processing
                if tx.send(frame).is_err() {
                    break;
                }
            }
        }));

        Ok(Self {
            cam_thread,
            keep_running,
        })
    }

    pub fn shutdown(&mut self) {
        tracing::warn!("Shutting down camera thread...");

        self.keep_running.store(false, Ordering::SeqCst);
        match self.cam_thread.take() {
            Some(cam_thread) => match cam_thread.join() {
                Ok(_) => (),
                Err(error) => {
                    tracing::error!("Unable to join camera thread : {:?}", error);
                    // panic!("Unable to join camera thread : {:?}", error);
                }
            },
            None => {
                tracing::error!("Called shutdown on a camera thread that was already shutdown");
                // panic!("Called shutdown on a camera thread that was already shutdown");
            }
        }
    }
}

#[test]
#[ignore = "Can only test this offline since it requires webcam, run cargo test -- --ignored"]
pub fn test_threaded_camera() -> Result<()> {
    let (tx, rx) = crossbeam_channel::unbounded::<Mat>();

    println!("{:?}", ThreadedCamera::get_available_cameras());

    let mut thr_cam = ThreadedCamera::start_camera_thread(tx, 0, "Default Camera".to_owned())?;

    for _ in 0..100 {
        let _frame = rx.recv()?;
    }

    thr_cam.shutdown();

    Ok(())
}
