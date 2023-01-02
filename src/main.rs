// #![windows_subsystem = "windows"]

mod camera;
mod enums;
mod filter;
mod gui;
mod network;
mod pose;
mod structs;
mod tddfa;
mod utils;

use crate::gui::style::{APP_NAME, APP_VERSION};
use crate::structs::{
    app::{AtomicF32, HeadTracker},
    camera::ThreadedCamera,
    config::AppConfig,
};
use iced::{
    window::{self, Icon},
    Application, Settings,
};
use std::sync::atomic::AtomicU32;
use std::sync::{atomic::AtomicBool, Arc};
use tracing::Level;

fn main() -> iced::Result {
    // let cfg: AppConfig = confy::load(APP_NAME, "config").unwrap();
    // confy::store(APP_NAME, "config", &AppConfig::default()).unwrap();

    let file_appender = tracing_appender::rolling::never(
        // ? Adding organization name
        directories::ProjectDirs::from("rs", "", APP_NAME)
            .unwrap()
            .data_dir()
            .to_str()
            .unwrap(), // * Similar path is also used by confy https://github.com/rust-cli/confy/blob/master/src/lib.rs#L316
        "logs.txt",
    );
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_max_level(Level::INFO)
        .init(); // ! Need to have only 1 log file which resets daily

    tracing::info!("Version {} on {}", APP_VERSION, std::env::consts::OS);
    tracing::info!(
        "The configuration file path is: {:#?}",
        confy::get_configuration_file_path(APP_NAME, "config").unwrap()
    );
    tracing::info!("The logs file name is: {}", "logs.txt");
    // tracing::info!("Config : {:#?}", cfg);

    HeadTracker::run(Settings {
        id: None,
        window: window::Settings {
            size: (750, 620), // start size
            position: window::Position::Centered,
            min_size: Some((750, 620)), // min size allowed
            max_size: None,
            resizable: true,
            decorations: true,
            transparent: false,
            always_on_top: false,
            // icon: Some(Icon::from_file_data(include_bytes!("../wix/Product.ico"), None).unwrap()),
            icon: None,
            visible: true,
        },
        flags: HeadTracker {
            min_cutoff: Arc::new(AtomicF32::new(0.0025)),
            beta: Arc::new(AtomicF32::new(0.01)),

            ip_arr_0: "127".to_string(),
            ip_arr_1: "0".to_string(),
            ip_arr_2: "0".to_string(),
            ip_arr_3: "1".to_string(),
            port: "4242".to_string(),

            fps: Arc::new(AtomicU32::new(60)),
            camera_list: ThreadedCamera::get_available_cameras().unwrap(),
            selected_camera: ThreadedCamera::get_available_cameras()
                .unwrap()
                .keys()
                .next()
                .cloned(),
            hide_camera: true,

            headtracker_thread: None,
            run_headtracker: Arc::new(AtomicBool::new(false)),
            should_exit: false,
        },
        default_font: None,
        default_text_size: 16,
        text_multithreading: false,
        antialiasing: false,
        exit_on_close_request: false,
        try_opengles_first: false,
    })
}
