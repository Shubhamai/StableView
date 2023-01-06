/// Saving state of the application
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};

use crate::{
    consts::APP_NAME,
    structs::app::{AtomicF32, Config, HeadTracker},
};

use serde::{Deserialize, Serialize};

use super::camera::ThreadedCamera;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct AppConfig {
    pub ip: String,
    pub port: String,
    pub min_cutoff: f32,
    pub beta: f32,
    pub fps: u32,
    pub selected_camera: String,
    pub hide_camera: bool,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            min_cutoff: 0.0025,
            beta: 0.01,

            ip: "127.0.0.1".to_string(),
            port: "4242".to_string(),

            fps: 60,

            selected_camera: ThreadedCamera::get_available_cameras()
                .unwrap()
                .keys()
                .next()
                .cloned()
                .unwrap(),

            hide_camera: true,
        }
    }
}

impl HeadTracker {
    pub fn load_config(&mut self) -> Config {
        let cfg: AppConfig = confy::load(APP_NAME, "config").unwrap(); // ! Error occurs when config data types in file does match config data types in code

        Config {
            min_cutoff: Arc::new(AtomicF32::new(cfg.min_cutoff)),
            beta: Arc::new(AtomicF32::new(cfg.beta)),

            ip: cfg.ip.to_string(),
            port: cfg.port.to_string(),

            fps: Arc::new(AtomicU32::new(cfg.fps)),

            selected_camera: Some(cfg.selected_camera),
            hide_camera: cfg.hide_camera,
        }
    }
    pub fn save_config(&self) {
        let config = AppConfig {
            ip: self.config.ip.clone(),
            port: self.config.port.clone(),
            min_cutoff: self.config.min_cutoff.load(Ordering::SeqCst),
            beta: self.config.beta.load(Ordering::SeqCst),
            fps: self.config.fps.load(Ordering::SeqCst),
            selected_camera: self
                .config
                .selected_camera
                .clone()
                .unwrap_or("No Camera Selected".to_string())
                .clone(),
            hide_camera: self.config.hide_camera,
        };

        confy::store(APP_NAME, "config", config).unwrap();
    }
}
