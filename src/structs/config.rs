// * Saving state of the application

use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AppConfig {
    pub ip_addr: (u8, u8, u8, u8),
    pub port: u16,
    pub min_cutoff: f32,
    pub beta: f32,
    pub fps: i32,
    pub default_camera_index: i32,
}
