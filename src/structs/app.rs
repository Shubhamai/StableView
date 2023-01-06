// Importing Modules
use std::{
    collections::HashMap,
    sync::{
        self,
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc,
    },
    thread,
};

use crate::gui::style::APP_VERSION;

use super::{camera::ThreadedCamera, config::AppConfig};

// * Adding this to another struct file
pub struct AtomicF32 {
    storage: AtomicU32,
}
impl AtomicF32 {
    pub fn new(value: f32) -> Self {
        let as_u64 = value.to_bits();
        Self {
            storage: AtomicU32::new(as_u64),
        }
    }
    pub fn store(&self, value: f32, ordering: Ordering) {
        let as_u64 = value.to_bits();
        self.storage.store(as_u64, ordering)
    }
    pub fn load(&self, ordering: Ordering) -> f32 {
        let as_u64 = self.storage.load(ordering);
        f32::from_bits(as_u64)
    }
}

pub struct HeadTracker {
    pub min_cutoff: sync::Arc<AtomicF32>,
    pub beta: sync::Arc<AtomicF32>,

    pub ip: String,
    pub port: String,

    pub fps: sync::Arc<AtomicU32>,

    pub camera_list: HashMap<String, i32>,
    pub selected_camera: Option<String>,
    pub hide_camera: bool,

    pub headtracker_thread: Option<thread::JoinHandle<String>>,
    pub headtracker_running: sync::Arc<AtomicBool>,

    pub should_exit: bool,
    pub error_message: Option<String>,

    version: String,
}

impl Default for HeadTracker {
    fn default() -> Self {
        HeadTracker {
            // ? Adding log directory path might lead to un-anonymous logs
            min_cutoff: Arc::new(AtomicF32::new(0.0025)),
            beta: Arc::new(AtomicF32::new(0.01)),

            ip: "127.0.0.1".to_string(),
            port: "4242".to_string(),

            fps: Arc::new(AtomicU32::new(60)),
            camera_list: ThreadedCamera::get_available_cameras().unwrap(), // ? Checking for new camera every 5 seconds ?
            selected_camera: ThreadedCamera::get_available_cameras() // ? Maybe checking for new cameras in main.rs
                .unwrap()
                .keys()
                .next()
                .cloned(),
            hide_camera: true,

            headtracker_thread: None,
            headtracker_running: Arc::new(AtomicBool::new(false)),

            should_exit: false,
            error_message: None,

            version: APP_VERSION.to_string(),
        }
    }
}
