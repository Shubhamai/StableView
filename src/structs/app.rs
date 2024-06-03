// Importing Modules
use std::{
    collections::HashMap,
    sync::{
        self,
        atomic::{AtomicBool, AtomicU32, Ordering},
        Arc, Mutex,
    },
    thread,
};

use crossbeam_channel::{unbounded, Receiver, Sender};
use opencv::{core::MatTraitConst, imgcodecs, prelude::Mat};

use super::{camera::ThreadedCamera, release::Release, state::AppConfig};
use crate::consts::{APP_GITHUB_API, APP_VERSION, NO_VIDEO_IMG};
use version_compare::{compare_to, Cmp};

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

#[derive(Clone)]
pub struct Config {
    pub min_cutoff: Arc<AtomicF32>,
    pub beta: Arc<AtomicF32>,

    pub ip: String,
    pub port: String,

    pub fps: Arc<AtomicU32>,

    pub selected_camera: String,
    pub hide_camera: bool,
}

// Contains configuration and state of the application and other data
pub struct HeadTracker {
    pub config: Config,

    pub camera_list: HashMap<String, i32>,

    pub headtracker_thread: Option<thread::JoinHandle<()>>,
    pub headtracker_running: sync::Arc<AtomicBool>,

    pub should_exit: bool,
    pub error_tracker: Arc<Mutex<String>>,

    pub sender: Sender<Mat>,
    pub receiver: Receiver<Mat>,
    pub frame: Mat,

    pub release_info: Option<Release>,
    pub version: String,
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

            selected_camera: AppConfig::default().selected_camera, // ? Maybe checking for new cameras in main.rs
            hide_camera: AppConfig::default().hide_camera,
        }
    }
}

impl Default for HeadTracker {
    fn default() -> Self {
        // Setup channels for camera thread to headtracker thread
        let (sender, receiver) = unbounded::<Mat>(); // ! bounded causes unwanted crashes bounded::<Mat>(1);

        let frame = match Mat::from_slice(NO_VIDEO_IMG) {
            Ok(frame) => frame.try_clone().unwrap(),
            Err(e) => {
                tracing::error!("Error loading NO_VIDEO_IMG: {}", e);
                Mat::default()
            }
        };
        let frame = match imgcodecs::imdecode(&frame, 1) {
            Ok(frame) => frame,
            Err(e) => {
                tracing::error!("Error decoding NO_VIDEO_IMG: {}", e);
                Mat::default()
            }
        };

        // Check for new version on GitHub
        let client = reqwest::blocking::Client::new();

        let response_json = match client
            .get(APP_GITHUB_API)
            .header("User-Agent", "rust-app")
            .send()
            .and_then(|response| response.json::<Release>())
        {
            Ok(release_info) => {
                if compare_to(&release_info.tag_name[1..], APP_VERSION, Cmp::Gt) == Ok(true) {
                    tracing::info!(
                        "New version available: {} (current: {})",
                        release_info.tag_name,
                        APP_VERSION
                    );
                    Some(release_info)
                } else {
                    None
                }
            }

            Err(e) => {
                tracing::info!("Unable to check for new version: {}", e);
                None
            }
        };

        HeadTracker {
            config: Config::default(),

            camera_list: match ThreadedCamera::get_available_cameras() {
                Ok(camera_list) => camera_list,
                Err(e) => {
                    tracing::error!("{}", e);
                    HashMap::new()
                }
            },

            headtracker_thread: None,
            headtracker_running: Arc::new(AtomicBool::new(false)),

            should_exit: false,
            error_tracker: Arc::new(Mutex::new(String::new())),

            version: APP_VERSION.to_string(),
            release_info: response_json,

            sender,
            receiver,
            frame,
        }
    }
}

impl std::fmt::Display for Config {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "(min_cutoff : {}, beta: {}, ip: {}, port: {}, fps: {}, selected_camera: {}, hide_camera: {})", 
        self.min_cutoff.load(Ordering::SeqCst), self.beta.load(Ordering::SeqCst), self.ip,self.port, self.fps.load(Ordering::SeqCst), self.selected_camera.clone(), self.hide_camera)
    }
}

impl std::fmt::Display for HeadTracker {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "(config: {}, camera_list: {:?}, headtracker_running: {}, should_exit: {}, version: {})", self.config, self.camera_list, self.headtracker_running.load(Ordering::SeqCst), self.should_exit, self.version)
    }
}
