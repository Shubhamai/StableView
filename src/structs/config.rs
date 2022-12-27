// * Saving state of the application

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
pub struct AppConfig {
    pub log_filename: String,
    pub ip_addr: (u8, u8, u8, u8),
    pub port: u16,
    pub min_cutoff: f32,
    pub beta: f32,
    pub fps: i32,
    pub default_camera_index: i32,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            // ? Adding log directory path might lead to un-anonymous logs
            log_filename: "logs.txt".to_string(),
            ip_addr: (127, 0, 0, 1),
            port: 4242,
            min_cutoff: 0.0025,
            beta: 0.01,
            fps: 60,
            default_camera_index: 0,
        }
    }
}
