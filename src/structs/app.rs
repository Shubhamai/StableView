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

use super::{camera::ThreadedCamera, state::AppConfig};

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

#[derive(Clone)] //Serialize, Deserialize, Debug,
pub struct Config {
    pub min_cutoff: Arc<AtomicF32>,
    pub beta: Arc<AtomicF32>,

    pub ip: String,
    pub port: String,

    pub fps: Arc<AtomicU32>,

    pub selected_camera: Option<String>,
    pub hide_camera: bool,
}

pub struct HeadTracker {
    pub config: Config,

    pub camera_list: HashMap<String, i32>,

    pub headtracker_thread: Option<thread::JoinHandle<String>>,
    pub headtracker_running: sync::Arc<AtomicBool>,

    pub should_exit: bool,
    pub error_message: Option<String>,

    version: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            // ? Adding log directory path might lead to un-anonymous logs
            min_cutoff: Arc::new(AtomicF32::new(AppConfig::default().min_cutoff)),
            beta: Arc::new(AtomicF32::new(AppConfig::default().beta)),

            ip: AppConfig::default().ip,
            port: AppConfig::default().port,

            fps: Arc::new(AtomicU32::new(AppConfig::default().fps)),

            selected_camera: Some(AppConfig::default().selected_camera), // ? Maybe checking for new cameras in main.rs
            hide_camera: AppConfig::default().hide_camera,
        }
    }
}

impl Default for HeadTracker {
    fn default() -> Self {
        HeadTracker {
            config: Config::default(),

            camera_list: ThreadedCamera::get_available_cameras().unwrap(), // ? Checking for new camera every 5 seconds ?

            headtracker_thread: None,
            headtracker_running: Arc::new(AtomicBool::new(false)),

            should_exit: false,
            error_message: None,

            version: APP_VERSION.to_string(),
        }
    }
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "(min_cutoff : {}, beta: {}, ip: {}, port: {}, fps: {}, selected_camera: {}, hide_camera: {})", 
        self.min_cutoff.load(Ordering::SeqCst), self.beta.load(Ordering::SeqCst), self.ip,self.port, self.fps.load(Ordering::SeqCst), self.selected_camera.clone().unwrap_or("No Camera".to_string()), self.hide_camera)
    }
}

impl std::fmt::Display for HeadTracker {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "(config: {}, camera_list: {:?}, headtracker_running: {}, should_exit: {}, error_message: {}, version: {})", self.config, self.camera_list, self.headtracker_running.load(Ordering::SeqCst), self.should_exit, self.error_message.clone().unwrap_or("No error message".to_string()), self.version)
    }
}
