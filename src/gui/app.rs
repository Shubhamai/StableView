use std::{
    sync::atomic::Ordering,
    thread,
    time::{Duration, Instant},
};

use iced::{
    application, executor, theme, widget::Container, window as iced_window, Application, Color,
    Command, Element, Length, Subscription, Theme,
};
use iced_native::{window, Event};

use crate::{
    enums::message::Message,
    filter::EuroDataFilter,
    structs::{app::HeadTracker, state::AppConfig},
    structs::{camera::ThreadedCamera, network::SocketNetwork, pose::ProcessHeadPose},
};

use crate::gui::view::run_page;

use crate::consts::{APP_NAME, APP_REPOSITORY};

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
        // match self.config.hide_camera {
        //     true =>
        iced_native::subscription::events().map(Message::EventOccurred) //,
                                                                        //     false => {
                                                                        //         if self.headtracker_running.load(Ordering::SeqCst) {
                                                                        //             let ticks = iced::time::every(Duration::from_millis(1)).map(|_| Message::Tick);
                                                                        //             let runtime_events =
                                                                        //                 iced_native::subscription::events().map(Message::EventOccurred);
                                                                        //             Subscription::batch(vec![runtime_events, ticks])
                                                                        //         } else {
                                                                        //             iced_native::subscription::events().map(Message::EventOccurred)
                                                                        //         }
                                                                        //     }
                                                                        // }
    }

    // fn should_exit(&self) -> bool {
    // self.should_exit
    // }

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
                        .unwrap(); // ! Error occures when default camera (on app installation) isn't changed
                    let config = self.config.clone();

                    let headtracker_running = self.headtracker_running.clone();
                    // let error_tracker = self.error_tracker.clone();

                    let tx = self.sender.clone();
                    let rx = self.receiver.clone();

                    self.headtracker_thread = Some(thread::spawn(move || {
                        let mut error_message = String::new();

                        let mut euro_filter = EuroDataFilter::new(
                            config.min_cutoff.load(Ordering::SeqCst),
                            config.beta.load(Ordering::SeqCst),
                        );
                        let mut socket_network = SocketNetwork::new(config.ip, config.port);

                        // Create a channel to communicate between threads

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

                            let out = head_pose.single_iter(&frame);

                            match out {
                                Ok(value) => {
                                    data = value;
                                }
                                Err(e) => {
                                    // println!("An error: {}; skipped.", e);
                                    // head_pose.face_box =  [150., 150., 400., 400.];
                                    // head_pose.pts_3d =
                                    //     vec![vec![1., 2., 3.], vec![4., 5., 6.], vec![7., 8., 9.]];
                                    // head_pose.face_box = [0., 0., 600., 600.];
                                    // headtracker_running.store(false, Ordering::SeqCst);
                                    continue;
                                }
                            };

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
                    self.headtracker_thread
                        .take()
                        .expect("Called stop on non-running thread")
                        .join()
                        .expect("Could not join spawned thread");
                }
            }
            Message::Tick => {
                self.frame = match self.receiver.try_recv() {
                    Ok(result) => result,
                    Err(_) => self.frame.clone(),
                };
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

                // If camera changes while running
                if self.headtracker_running.load(Ordering::SeqCst) {
                    // Turn it back off and on again :)
                    self.update(Message::Toggle);
                    self.update(Message::Toggle);
                }

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
                if let Event::Window(window::Event::CloseRequested) = event {
                    if self.headtracker_running.load(Ordering::SeqCst) {
                        self.headtracker_running.store(false, Ordering::SeqCst);
                        self.headtracker_thread
                            .take()
                            .expect("Called stop on non-running thread")
                            .join()
                            .expect("Could not join spawned thread");
                    }
                    // self.should_exit = true;
                    std::process::exit(0);
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
