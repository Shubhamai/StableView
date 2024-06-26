#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(non_snake_case)]

mod camera;
mod consts;
mod enums;
mod face;
mod filter;
mod gui;
mod network;
mod process;
mod structs;
mod tddfa;
mod utils;

use crate::{
    consts::{APP_NAME, APP_VERSION, ICON, INTER_FONT},
    structs::app::HeadTracker,
};
use consts::ICONS_FONT;
use iced::{
    window::{self, settings::PlatformSpecific, Level},
    Application, Settings, Size,
};
use image::ImageFormat;

use std::{fs, path::Path};

use anyhow::Result;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // ? Adding organization name
    let log_filepath = match directories::ProjectDirs::from("rs", "", APP_NAME) {
        Some(dirs) => dirs,
        None => {
            tracing::error!("Could not find project directories");
            std::process::exit(1);
        }
    };
    let log_filename = "StableView.log";

    // println!("{:?}", std::env::current_exe().unwrap());

    match fs::remove_file(log_filepath.data_dir().join(log_filename)) {
        Ok(_) => tracing::warn!("Removed old log file"),
        Err(_) => tracing::warn!("No old log file found"),
    }

    let file_appender = tracing_appender::rolling::never(
        // * Similar path is also used by confy https://github.com/rust-cli/confy/blob/master/src/lib.rs#L316
        match log_filepath.data_dir().to_str() {
            Some(path) => path,
            None => {
                tracing::error!("Could not find project directories");
                std::process::exit(1);
            }
        },
        log_filename,
    );

    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    tracing_subscriber::fmt()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_max_level(tracing::Level::WARN)
        .init(); // ! Need to have only 1 log file which resets daily

    tracing::warn!("Version {} on {}", APP_VERSION, std::env::consts::OS);
    tracing::warn!(
        "The configuration file path is: {:#?}",
        match confy::get_configuration_file_path(APP_NAME, "config") {
            Ok(path) => path,
            Err(e) => {
                tracing::error!("Error getting config file path: {}", e);
                Path::new("Could not find config file path").to_path_buf()
            }
        }
    );

    let mut flags = HeadTracker::default();
    flags.config = flags.load_config();

    tracing::warn!("Config : {}", flags);

    let settings = Settings {
        id: None,
        window: window::Settings {
            size: Size::new(750., 620.), // start size
            position: window::Position::Centered,
            min_size: Some(Size::new(750., 620.)),
            max_size: None,
            resizable: true,
            decorations: true,
            transparent: false,
            icon: match window::icon::from_file_data(ICON, Some(ImageFormat::Ico)) {
                Ok(icon) => Some(icon),
                Err(_) => None,
            },
            visible: true,
            platform_specific: PlatformSpecific::default(),
            exit_on_close_request: true,
            level: Level::Normal,
        
        },
        flags,
        default_font: iced::font::Font::with_name("Inter-Regular"),
        fonts: vec![
            std::borrow::Cow::Borrowed(INTER_FONT),
            std::borrow::Cow::Borrowed(ICONS_FONT),
        ],
        default_text_size: iced::Pixels(13.),
        antialiasing: false,
    };

    if let Err(e) = HeadTracker::run(settings) {
        tracing::error!("{}", e);
    }

    Ok(())
}
