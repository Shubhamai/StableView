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
use std::sync::mpsc;
use tracing::Level;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let fps = 60;

    let file_appender = tracing_appender::rolling::never("", "logs.txt");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_max_level(Level::INFO)
        .init(); // ! Need to have only 1 log file which resets daily

    tracing::info!("Version {}", env!("CARGO_PKG_VERSION"));
    tracing::info!("fps set to {fps}");

    let euro_filter = EuroDataFilter::new();
    let socket_network = SocketNetwork::new(4242);
    // Create a channel to communicate between threads
    let (tx, rx) = mpsc::channel();
    let mut thr_cam = ThreadedCamera::setup_camera();
    thr_cam.start_camera_thread(tx);
    let mut head_pose =
        ProcessHeadPose::new("./assets/data.json", "./assets/mb05_120x120.onnx", 120, 60);
    head_pose.start_loop(rx, euro_filter, socket_network);
    thr_cam.shutdown();
    // head_pose.shutdown();

    Ok(())
}
