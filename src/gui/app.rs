use std::{
    sync::{atomic::Ordering, mpsc},
    thread,
    time::{Duration, Instant},
};

use iced::{
    executor, widget::Container, Application, Command, Element, Length, Subscription, Theme,
};
use iced_native::{window, Event};

use crate::{
    enums::message::Message,
    filter::EuroDataFilter,
    structs::app::HeadTracker,
    structs::{camera::ThreadedCamera, network::SocketNetwork, pose::ProcessHeadPose},
};

use crate::gui::view::run_page;

use super::style::{APP_NAME, APP_REPOSITORY};

impl Application for HeadTracker {
    type Executor = executor::Default;
    type Flags = HeadTracker;
    type Message = Message;
    type Theme = Theme;

    fn new(flags: HeadTracker) -> (HeadTracker, Command<Message>) {
        (flags, Command::none())
    }

    fn title(&self) -> String {
        String::from(APP_NAME)
    }

    fn subscription(&self) -> Subscription<Message> {
        iced_native::subscription::events().map(Message::EventOccurred)
    }

    fn should_exit(&self) -> bool {
        self.should_exit
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Toggle => {
                if !self.run_headtracker.load(Ordering::SeqCst) {
                    self.run_headtracker.store(true, Ordering::SeqCst);

                    let min_cutoff = self.min_cutoff.clone();
                    let beta = self.beta.clone();
                    let (ip_arr_0, ip_arr_1, ip_arr_2, ip_arr_3, port) = (
                        self.ip_arr_0.clone(),
                        self.ip_arr_1.clone(),
                        self.ip_arr_2.clone(),
                        self.ip_arr_3.clone(),
                        self.port.clone(),
                    );
                    let camera_index = *self
                        .camera_list
                        .get(self.selected_camera.as_ref().unwrap())
                        .unwrap();
                    let fps = self.fps.clone();

                    let run_headtracker = self.run_headtracker.clone();

                    self.headtracker_thread = Some(thread::spawn(move || {
                        let mut euro_filter = EuroDataFilter::new(
                            min_cutoff.load(Ordering::SeqCst),
                            beta.load(Ordering::SeqCst),
                        );
                        let mut socket_network =
                            SocketNetwork::new(ip_arr_0, ip_arr_1, ip_arr_2, ip_arr_3, port);

                        // Create a channel to communicate between threads
                        let (tx, rx) = mpsc::channel();
                        let mut thr_cam = ThreadedCamera::start_camera_thread(tx, camera_index);

                        let mut head_pose = ProcessHeadPose::new(120);

                        let mut frame = rx.recv().unwrap();
                        let mut data;

                        while run_headtracker.load(Ordering::SeqCst) {
                            let start_time = Instant::now();

                            frame = match rx.try_recv() {
                                Ok(result) => result,
                                Err(_) => frame.clone(),
                            };

                            data = head_pose.single_iter(&frame);

                            data = euro_filter.filter_data(
                                data,
                                Some(min_cutoff.load(Ordering::SeqCst)),
                                Some(beta.load(Ordering::SeqCst)),
                            );

                            socket_network.send(data);

                            let elapsed_time = start_time.elapsed();
                            let delay_time = ((1000 / fps.load(Ordering::SeqCst)) as f32
                                - elapsed_time.as_millis() as f32)
                                .max(0.);
                            thread::sleep(Duration::from_millis(delay_time.round() as u64));
                        }

                        thr_cam.shutdown();
                    }));
                } else {
                    self.run_headtracker.store(false, Ordering::SeqCst);
                    self.headtracker_thread
                        .take()
                        .expect("Called stop on non-running thread")
                        .join()
                        .expect("Could not join spawned thread");
                }
            }
            Message::MinCutoffSliderChanged(value) => {
                if value == 0 {
                    self.min_cutoff.store(0., Ordering::SeqCst)
                } else {
                    self.min_cutoff
                        .store(1. / ((value * value) as f32), Ordering::SeqCst)
                }
            }
            Message::BetaSliderChanged(value) => {
                if value == 0 {
                    self.beta.store(0., Ordering::SeqCst)
                } else {
                    self.beta
                        .store(1. / ((value * value) as f32), Ordering::SeqCst)
                }
            }
            Message::FPSSliderChanged(fps) => self.fps.store(fps, Ordering::SeqCst),
            Message::InputIP0(ip_arr_0) => self.ip_arr_0 = ip_arr_0,
            Message::InputIP1(ip_arr_1) => self.ip_arr_1 = ip_arr_1,
            Message::InputIP2(ip_arr_2) => self.ip_arr_2 = ip_arr_2,
            Message::InputIP3(ip_arr_3) => self.ip_arr_3 = ip_arr_3,
            Message::InputPort(port) => self.port = port,
            Message::Camera(camera_name) => self.selected_camera = Some(camera_name),
            Message::HideCamera(value) => self.hide_camera = value,
            Message::OpenGithub => {
                #[cfg(target_os = "windows")]
                std::process::Command::new("explorer")
                    .arg(APP_REPOSITORY)
                    .spawn()
                    .unwrap();
                #[cfg(target_os = "macos")]
                std::process::Command::new("open")
                    .arg(APP_REPOSITORY)
                    .spawn()
                    .unwrap();
                #[cfg(target_os = "linux")]
                std::process::Command::new("xdg-open")
                    .arg(APP_REPOSITORY)
                    .spawn()
                    .unwrap();
            }
            Message::OpenLogs => {
                #[cfg(target_os = "windows")]
                std::process::Command::new("explorer")
                    .arg(
                        directories::ProjectDirs::from("rs", "", APP_NAME)
                            .unwrap()
                            .data_dir()
                            .to_str()
                            .unwrap(),
                    )
                    .spawn()
                    .unwrap();
                #[cfg(target_os = "macos")]
                std::process::Command::new("open")
                    .arg("-t")
                    .arg(
                        directories::ProjectDirs::from("rs", "", APP_NAME)
                            .unwrap()
                            .data_dir()
                            .to_str()
                            .unwrap(),
                    )
                    .spawn()
                    .unwrap();
                #[cfg(target_os = "linux")]
                std::process::Command::new("xdg-open")
                    .arg(
                        directories::ProjectDirs::from("rs", "", APP_NAME)
                            .unwrap()
                            .data_dir()
                            .to_str()
                            .unwrap(),
                    )
                    .spawn()
                    .unwrap();
            }
            Message::EventOccurred(event) => {
                if Event::Window(window::Event::CloseRequested) == event {
                    if self.run_headtracker.load(Ordering::SeqCst) {
                        self.run_headtracker.store(false, Ordering::SeqCst);
                        self.headtracker_thread
                            .take()
                            .expect("Called stop on non-running thread")
                            .join()
                            .expect("Could not join spawned thread");
                    }
                    // confy::store(APP_NAME, "config", self.cfg.clone()).unwrap();
                    self.should_exit = true;
                }
            }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        let body = run_page(self);

        Container::new(body)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
