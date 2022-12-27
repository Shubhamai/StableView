use std::{
    sync,
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc,
    },
    thread, time,
};

use iced::{
    alignment, executor,
    widget::{button, column, container, row, text},
    window::{self, Position},
    Alignment, Application, Command, Element, Length, Settings, Subscription, Theme,
};
use serde::{Deserialize, Serialize};

use crate::{
    camera::ThreadedCamera, filter::EuroDataFilter, network::SocketNetwork, pose::ProcessHeadPose,
    structs::headtracker::HeadTracker,
};

#[derive(Debug, Clone, Copy)]
pub enum Message {
    RunClicked,
    StopClicked,
    Toggle,
}

impl Application for HeadTracker {
    type Executor = executor::Default;
    type Flags = HeadTracker;
    type Message = Message;
    type Theme = Theme;

    fn new(flags: HeadTracker) -> (HeadTracker, Command<Message>) {
        (flags, Command::none())
    }

    fn title(&self) -> String {
        String::from("Head Tracker")
    }

    fn update(&mut self, message: Message) -> Command<Message> {
        match message {
            Message::Toggle => {
                if !self.keep_running.load(Ordering::SeqCst) {
                    self.keep_running.store(true, Ordering::SeqCst);
                    let cloned_keep_running = self.keep_running.clone();

                    thread::spawn(move || {
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
                    });
                } else {
                    self.keep_running.store(false, Ordering::SeqCst)
                    // println!("Thread already running")
                }
            }
            _ => {} // Message::StopClicked => {
                    //     if self.keep_running.load(Ordering::SeqCst) {
                    //         self.keep_running.store(false, Ordering::SeqCst)
                    //     } else {
                    //         println!("Thread already stopped")
                    //     }
                    // }
        }
        Command::none()
    }

    fn view(&self) -> Element<Message> {
        // column![
        //     button("Start").on_press(Message::RunClicked),
        //     button("Stop").on_press(Message::StopClicked)
        // ]
        // .padding(20)
        // .align_items(Alignment::Center)
        // .into()

        let button = |label| {
            button(text(label).horizontal_alignment(alignment::Horizontal::Center))
                .padding(10)
                .width(Length::Units(80))
        };
        let toggle_button = {
            let label = match self.keep_running.load(Ordering::SeqCst) {
                true => "Stop",
                false => "Start",
            };

            button(label).on_press(Message::Toggle)
        };

        let controls = row![toggle_button].spacing(20);

        let content = column![controls].align_items(Alignment::Center).spacing(20);

        container(content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into()
    }
}
