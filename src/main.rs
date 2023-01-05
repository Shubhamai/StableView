#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

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
use iced::{window, Application, Settings};
use std::sync::{
    atomic::{AtomicBool, AtomicU32},
    Arc,
};
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
            icon: Some(
                window::Icon::from_file_data(include_bytes!("../assets/brand/Product.ico"), None)
                    .unwrap(),
            ),
            visible: true,
        },
        flags: HeadTracker::default(),
        default_font: Some(include_bytes!("../assets/fonts/Inter-Regular.ttf")),
        default_text_size: 16,
        text_multithreading: false,
        antialiasing: false,
        exit_on_close_request: false,
        try_opengles_first: false,
    })
}
