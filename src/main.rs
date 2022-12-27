// #![windows_subsystem = "windows"]

mod camera;
mod filter;
mod model;
mod network;
mod pose;
mod tddfa;
// mod utils;
mod enums;
mod structs;
mod utils;
mod gui;

use crate::filter::EuroDataFilter;
use crate::pose::ProcessHeadPose;

use camera::ThreadedCamera;
use network::SocketNetwork;
// use serde::{Deserialize, Serialize};
use std::{
    sync::{
        self,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Sender},
    },
    thread,
};
use tracing::Level;

// use iced::executor;
// use iced::{Application, Command, Element, Settings, Theme};
use iced::{
    executor,
    widget::{button, column, text},
    window::{self, Position},
    Alignment, Application, Command, Element, Length, Settings, Subscription, Theme,
};
// use iced::{widget::{button, column, text}, Command, Application};
// use iced::{Alignment, Element, Sandbox, Settings};

use crate::structs::{config::AppConfig, headtracker::HeadTracker};

fn main() -> iced::Result {
    let cfg: AppConfig = confy::load(env!("CARGO_PKG_NAME"), "config").unwrap();
    let cfg_filepath =
        confy::get_configuration_file_path(env!("CARGO_PKG_NAME"), "config").unwrap();
    confy::store(env!("CARGO_PKG_NAME"), "config", &AppConfig::default()).unwrap();

    let file_appender = tracing_appender::rolling::never(
        directories::ProjectDirs::from("rs", "", env!("CARGO_PKG_NAME"))
            .unwrap()
            .data_dir()
            .to_str()
            .unwrap(), // * Similar path is also used by confy https://github.com/rust-cli/confy/blob/master/src/lib.rs#L316
        cfg.log_filename.clone(),
    );
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
    tracing::info!("The logs file name is: {}", cfg.log_filename);
    tracing::info!("Config : {:#?}", cfg);

    let filter = EuroDataFilter::new(cfg.min_cutoff, cfg.beta);
    let socket = SocketNetwork::new(cfg.ip_addr, cfg.port);

    // // Create a channel to communicate between threads
    // let (tx, rx) = mpsc::channel();
    let camera_index = 0;
    // let camera = ThreadedCamera::start_camera_thread(tx, );

    let head_pose = ProcessHeadPose::new(
        "./assets/data.json",
        "./assets/mb05_120x120.onnx",
        120,
        cfg.fps as u128,
    );
    // head_pose.start_loop(rx, euro_filter, socket_network);

    // thr_cam.shutdown();

    // Ok(())

    HeadTracker::run(Settings {
        id: None,
        window: window::Settings {
            size: (768, 716), // start size
            position: Position::Centered,
            min_size: Some((768, 716)), // min size allowed
            max_size: None,
            resizable: true,
            decorations: true,
            transparent: false,
            always_on_top: false,
            icon: None,
            visible: true,
        },
        flags: HeadTracker {
            keep_running: sync::Arc::new(AtomicBool::new(false)),
            filter,
            socket,
            camera_index,
            // head_pose,
        },
        default_font: None,
        default_text_size: 16,
        text_multithreading: false,
        antialiasing: false,
        exit_on_close_request: true,
        try_opengles_first: false,
    })
}

// #[derive(Debug, Clone, Copy)]
// enum Message {
//     RunClicked,
//     StopClicked,
// }

// // struct HeadTracker {
// //     // run_thread: Option<thread::JoinHandle<()>>,
// //     keep_running: sync::Arc<AtomicBool>,
// // }

// impl Application for HeadTracker {
//     // type Executor = executor::Default;
//     // type Flags = ();
//     // type Message = ();
//     // type Theme = Theme;
//     // type Message = Message;

//     // fn new() -> HeadTracker {
//     //     let keep_running = sync::Arc::new(AtomicBool::new(false));

//     //     HeadTracker { keep_running }
//     // }

//     type Executor = executor::Default;
//     type Flags = HeadTracker;
//     type Message = Message;
//     type Theme = Theme;

//     fn new(flags: HeadTracker) -> (HeadTracker, Command<Message>) {
//         // let keep_running = sync::Arc::new(AtomicBool::new(false));
//         (flags, Command::none())
//     }

//     fn title(&self) -> String {
//         String::from("Head Tracker")
//     }

//     fn update(&mut self, message: Message) -> Command<Message> {
//         match message {
//             Message::RunClicked => {
//                 self.keep_running.store(true, Ordering::SeqCst);
//                 let cloned_keep_running = self.keep_running.clone();

//                 thread::spawn(move || {
//                     let mut euro_filter = EuroDataFilter::new(0.0025, 0.01);
//                     let mut socket_network = SocketNetwork::new((127, 0, 0, 1), 4242);

//                     // Create a channel to communicate between threads
//                     let (tx, rx) = mpsc::channel();
//                     let mut thr_cam = ThreadedCamera::start_camera_thread(tx, 0);

//                     let mut head_pose = ProcessHeadPose::new(
//                         "./assets/data.json",
//                         "./assets/mb05_120x120.onnx",
//                         120,
//                         60,
//                     );

//                     let mut frame = rx.recv().unwrap();
//                     let mut data;

//                     while cloned_keep_running.load(Ordering::SeqCst) {
//                         frame = match rx.try_recv() {
//                             Ok(result) => result,
//                             Err(_) => frame.clone(),
//                         };

//                         data = head_pose.single_iter(&frame);

//                         data = euro_filter.filter_data(data);

//                         socket_network.send(data);
//                     }

//                     thr_cam.shutdown();
//                 });
//             }
//             Message::StopClicked => self.keep_running.store(false, Ordering::SeqCst),
//         }
//         Command::none()
//     }

//     fn view(&self) -> Element<Message> {
//         column![
//             button("Start").on_press(Message::RunClicked),
//             // text(self.value).size(50),
//             button("Stop").on_press(Message::StopClicked)
//         ]
//         .padding(20)
//         .align_items(Alignment::Center)
//         .into()
//     }
// }
