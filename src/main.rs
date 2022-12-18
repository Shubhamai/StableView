// #![windows_subsystem = "windows"]

mod camera;
mod filter;
mod model;
mod network;
mod pose;
mod tddfa;
mod utils;

use crate::filter::EuroDataFilter;
use crate::pose::ProcessHeadPose;

use camera::ThreadedCamera;
use network::SocketNetwork;
use serde::{Deserialize, Serialize};
use std::{io::Read, sync::mpsc};
use tracing::Level;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
struct AppConfig {
    log_directory: String,
    log_filename: String,
    ip_addr: (u8, u8, u8, u8),
    port: u16,
    min_cutoff: f32,
    beta: f32,
    fps: i32,
    default_camera_index: i32,
}

impl Default for AppConfig {
    fn default() -> Self {
        AppConfig {
            log_directory: directories::ProjectDirs::from("rs", "", env!("CARGO_PKG_NAME"))
                .unwrap()
                .data_dir()
                .to_str()
                .unwrap()
                .to_owned(),
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

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cfg: AppConfig = confy::load(env!("CARGO_PKG_NAME"), "config").unwrap();
    let cfg_filepath = confy::get_configuration_file_path(env!("CARGO_PKG_NAME"), "config")?;
    confy::store(env!("CARGO_PKG_NAME"), "config", &AppConfig::default())?;

    let file_appender =
        tracing_appender::rolling::never(cfg.log_directory.clone(), cfg.log_filename.clone());
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_max_level(Level::INFO)
        .init(); // ! Need to have only 1 log file which resets daily

    tracing::info!(
        "Version {} on {}",
        env!("CARGO_PKG_VERSION"),
        std::env::consts::OS
    );

    tracing::info!("The configuration file path is: {:#?}", cfg_filepath);
    tracing::info!(
        "The logs file path is: {} as {}",
        cfg.log_directory,
        cfg.log_filename
    );
    tracing::info!("Config : {:#?}", cfg);

    let euro_filter = EuroDataFilter::new(cfg.min_cutoff, cfg.beta);
    let socket_network = SocketNetwork::new(cfg.ip_addr, cfg.port);

    // Create a channel to communicate between threads
    let (tx, rx) = mpsc::channel();
    let mut thr_cam = ThreadedCamera::start_camera_thread(tx, cfg.default_camera_index);

    let mut head_pose = ProcessHeadPose::new(
        "./assets/data.json",
        "./assets/mb05_120x120.onnx",
        120,
        cfg.fps as u128,
    );
    head_pose.start_loop(rx, euro_filter, socket_network);

    thr_cam.shutdown();

    Ok(())
}
