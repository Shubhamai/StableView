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

use iced::widget::{button, column, text};
use iced::{Alignment, Element, Sandbox, Settings};

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(default)]
struct AppConfig {
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
            .unwrap()
            .to_owned(), // * Similar path is also used by confy https://github.com/rust-cli/confy/blob/master/src/lib.rs#L316
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

    // let euro_filter = EuroDataFilter::new(cfg.min_cutoff, cfg.beta);
    // let socket_network = SocketNetwork::new(cfg.ip_addr, cfg.port);

    // // Create a channel to communicate between threads
    // let (tx, rx) = mpsc::channel();
    // let mut thr_cam = ThreadedCamera::start_camera_thread(tx, cfg.default_camera_index);

    // let mut head_pose = ProcessHeadPose::new(
    //     "./assets/data.json",
    //     "./assets/mb05_120x120.onnx",
    //     120,
    //     cfg.fps as u128,
    // );
    // head_pose.start_loop(rx, euro_filter, socket_network);

    // thr_cam.shutdown();

    // Ok(())

    Hello::run(Settings::default())
}

#[derive(Debug, Clone, Copy)]
enum Message {
    RunClicked,
    StopClicked,
}

struct Hello {
    // run_thread: Option<thread::JoinHandle<()>>,
    keep_running: sync::Arc<AtomicBool>,
}

impl Sandbox for Hello {
    // type Executor = executor::Default;
    // type Flags = ();
    // type Message = ();
    // type Theme = Theme;
    type Message = Message;

    fn new() -> Hello {
        let keep_running = sync::Arc::new(AtomicBool::new(false));

        Hello { keep_running }
    }

    fn title(&self) -> String {
        String::from("Head Tracker")
    }

    fn update(&mut self, message: Message) {
        match message {
            Message::RunClicked => {
                self.keep_running.store(true, Ordering::SeqCst);
                let cloned_keep_running = self.keep_running.clone();

                Some(thread::spawn(move || {
                    let mut euro_filter = EuroDataFilter::new(0.0025, 0.01);
                    let mut socket_network = SocketNetwork::new((127, 0, 0, 1), 4242);

                    // Create a channel to communicate between threads
                    let (tx, rx) = mpsc::channel();
                    let mut thr_cam = ThreadedCamera::start_camera_thread(tx, 0);

                    let mut head_pose = ProcessHeadPose::new(
                        "./assets/data.json",
                        "./assets/mb05_120x120.onnx",
                        120,
                        60,
                    );

                    let mut frame = rx.recv().unwrap();
                    let mut data;

                    while cloned_keep_running.load(Ordering::SeqCst) {
                        frame = match rx.try_recv() {
                            Ok(result) => result,
                            Err(_) => frame.clone(),
                        };

                        data = head_pose.single_iter(&frame);

                        data = euro_filter.filter_data(data);

                        socket_network.send(data);
                    }

                    thr_cam.shutdown();
                }));
            }
            Message::StopClicked => self.keep_running.store(false, Ordering::SeqCst),
        }
    }

    fn view(&self) -> Element<Message> {
        column![
            button("Start").on_press(Message::RunClicked),
            // text(self.value).size(50),
            button("Stop").on_press(Message::StopClicked)
        ]
        .padding(20)
        .align_items(Alignment::Center)
        .into()
    }
}
