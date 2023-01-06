use std::{
    sync::{atomic::Ordering, mpsc},
    thread,
    time::{Duration, Instant},
};

use iced::{
    application, executor, theme, widget::Container, Application, Color, Command, Element, Length,
    Subscription, Theme,
};
use iced_native::{window, Event};

use crate::{
    enums::message::Message,
    filter::EuroDataFilter,
    structs::{app::HeadTracker, state::AppConfig},
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

    fn theme(&self) -> Theme {
        Theme::Light
    }

    fn style(&self) -> theme::Application {
        fn dark_background(_theme: &Theme) -> application::Appearance {
            application::Appearance {
                background_color: Color::from_rgb8(245, 245, 245),
                text_color: Color::BLACK,
            }
        }

        theme::Application::from(dark_background as fn(&Theme) -> _)
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Toggle => {
                if !self.headtracker_running.load(Ordering::SeqCst) {
                    self.headtracker_running.store(true, Ordering::SeqCst);

                    let camera_index = *self
                        .camera_list
                        .get(self.config.selected_camera.as_ref().unwrap())
                        .unwrap();
                    let config = self.config.clone();

                    let headtracker_running = self.headtracker_running.clone();

                    self.headtracker_thread = Some(thread::spawn(move || {
                        let mut error_message: String = "".to_owned();

                        let mut euro_filter = EuroDataFilter::new(
                            config.min_cutoff.load(Ordering::SeqCst),
                            config.beta.load(Ordering::SeqCst),
                        );
                        let mut socket_network = SocketNetwork::new(config.ip, config.port);

                        // Create a channel to communicate between threads
                        let (tx, rx) = mpsc::channel();
                        let mut thr_cam = ThreadedCamera::start_camera_thread(tx, camera_index);

                        let mut head_pose = ProcessHeadPose::new(120);

                        let mut frame = rx.recv().unwrap();
                        let mut data;

                        while headtracker_running.load(Ordering::SeqCst) {
                            let start_time = Instant::now();

                            frame = match rx.try_recv() {
                                Ok(result) => result,
                                Err(_) => frame.clone(),
                            };

                            data = head_pose.single_iter(&frame);

                            data = euro_filter.filter_data(
                                data,
                                Some(config.min_cutoff.load(Ordering::SeqCst)),
                                Some(config.beta.load(Ordering::SeqCst)),
                            );

                            match socket_network.send(data) {
                                Ok(_) => {}
                                Err(_) => {
                                    error_message = "Unable to send data".to_string();
                                    break;
                                }
                            };

                            let elapsed_time = start_time.elapsed();
                            let delay_time = ((1000 / config.fps.load(Ordering::SeqCst)) as f32
                                - elapsed_time.as_millis() as f32)
                                .max(0.);
                            thread::sleep(Duration::from_millis(delay_time.round() as u64));
                        }

                        thr_cam.shutdown();

                        headtracker_running.store(false, Ordering::SeqCst);
                        error_message
                    }));
                } else {
                    self.headtracker_running.store(false, Ordering::SeqCst);
                    Some(
                        self.headtracker_thread
                            .take()
                            .expect("Called stop on non-running thread")
                            .join()
                            .expect("Could not join spawned thread"),
                    );
                }
            }
            Message::MinCutoffSliderChanged(value) => {
                if value == 0 {
                    self.config.min_cutoff.store(0., Ordering::SeqCst)
                } else {
                    self.config
                        .min_cutoff
                        .store(1. / ((value * value) as f32), Ordering::SeqCst)
                };
                self.save_config()
            }
            Message::BetaSliderChanged(value) => {
                if value == 0 {
                    self.config.beta.store(0., Ordering::SeqCst)
                } else {
                    self.config
                        .beta
                        .store(1. / ((value * value) as f32), Ordering::SeqCst)
                };
                self.save_config()
            }
            Message::FPSSliderChanged(fps) => {
                self.config.fps.store(fps, Ordering::SeqCst);
                self.save_config()
            }
            Message::InputIP(ip) => {
                self.config.ip = ip;
                self.save_config()
            } // ! Input validation, four decimal with respective numbers between
            Message::InputPort(port) => {
                self.config.port = port;
                self.save_config()
            } // ! Input validation, only numbers
            Message::Camera(camera_name) => {
                self.config.selected_camera = Some(camera_name);
                self.save_config()
            }
            Message::HideCamera(value) => {
                self.config.hide_camera = value;
                self.save_config()
            }
            // ! Need more asthetic default settings
            Message::DefaultSettings => {
                self.config
                    .min_cutoff
                    .store(AppConfig::default().min_cutoff, Ordering::SeqCst);
                self.config
                    .beta
                    .store(AppConfig::default().beta, Ordering::SeqCst);
                self.config
                    .fps
                    .store(AppConfig::default().fps, Ordering::SeqCst);
                self.config.ip = AppConfig::default().ip;
                self.config.port = AppConfig::default().port;
                self.config.hide_camera = AppConfig::default().hide_camera;

                self.save_config();
            }
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
                    if self.headtracker_running.load(Ordering::SeqCst) {
                        self.headtracker_running.store(false, Ordering::SeqCst);
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
