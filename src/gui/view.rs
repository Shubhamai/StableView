use std::{borrow::Cow, sync::atomic::Ordering};

use iced::{
    alignment::{self, Horizontal, Vertical},
    widget::{
        button, horizontal_space, pick_list, rule, slider, text, text_input, toggler,
        vertical_space, Column, Container, Row, Text,
    },
    Alignment, Length, Renderer,
};

use crate::{enums::message::Message, structs::app::HeadTracker};

use super::style::{APP_AUTHORS, APP_NAME, APP_VERSION, HEIGHT_BODY, HEIGHT_FOOTER};

pub fn run_page(headtracker: &HeadTracker) -> Column<Message> {
    let min_cutoff = {
        if headtracker.min_cutoff.load(Ordering::SeqCst) - 0. < f32::EPSILON {
            0
        } else {
            (1. / headtracker.min_cutoff.load(Ordering::SeqCst)).sqrt() as u32
        }
    };

    let beta = {
        if headtracker.beta.load(Ordering::SeqCst) - 0. < f32::EPSILON {
            0
        } else {
            (1. / headtracker.beta.load(Ordering::SeqCst)).sqrt() as u32
        }
    };
    let fps = headtracker.fps.load(Ordering::SeqCst);

    let ip = headtracker.ip.as_str();
    let port = headtracker.port.as_str();
    let hide_camera = headtracker.hide_camera;

    let min_cutoff_slider = slider(0..=50, min_cutoff, Message::MinCutoffSliderChanged).step(1);
    let beta_slider = slider(0..=50, beta, Message::BetaSliderChanged).step(1);
    let fps_slider = slider(15..=120, fps, Message::FPSSliderChanged).step(1);

    let toggle_start = {
        let label = match headtracker.headtracker_running.load(Ordering::SeqCst) {
            true => "Stop",
            false => "Start",
        };
        button(
            text(label)
                .vertical_alignment(Vertical::Center)
                .horizontal_alignment(Horizontal::Center),
        )
        .height(Length::Units(40))
        .width(Length::Units(180))
        .on_press(Message::Toggle)
    };

    let sliders_row = Container::new(
        Column::new()
            .push(text("Filter Settings").size(15))
            .push(vertical_space(Length::Units(20)))
            .push(text("Speed").size(14))
            .push(Container::new(min_cutoff_slider).width(Length::FillPortion(2)))
            .push(vertical_space(Length::Units(10)))
            .push(text("Smooth").size(14))
            .push(Container::new(beta_slider).width(Length::FillPortion(2)))
            .push(vertical_space(Length::Units(30)))
            .push(text("FPS").size(15))
            .push(Container::new(fps_slider).width(Length::FillPortion(2)))
            .push(vertical_space(Length::Units(30)))
            .push(text("IP and Port").size(15))
            // ! IPV4 and V6 support for external devices, having only two inputs, ip and port
            .push(Container::new(
                Row::new()
                    .spacing(5)
                    .push(
                        text_input("127.0.0.1", ip, Message::InputIP)
                            .width(Length::FillPortion(70)),
                    )
                    .push(text("      "))
                    .push(
                        text_input("4242", port, Message::InputPort).width(Length::FillPortion(15)),
                    ),
            ))
            .push(vertical_space(Length::Units(30))),
    )
    .padding(40);

    let camera_row = Container::new(
        Column::new()
            .push({
                iced::widget::image::viewer(iced::widget::image::Handle::from_path(
                    "assets/brand/no_video.png",
                ))
                .width(Length::Fill)
                .height(Length::Units(200))
            })
            .push(vertical_space(Length::Units(32)))
            .push(Container::new(
                Row::new()
                    .push(
                        pick_list(
                            Cow::from(
                                headtracker
                                    .camera_list
                                    .keys()
                                    .cloned()
                                    .collect::<Vec<String>>(),
                            ),
                            headtracker.selected_camera.clone(),
                            Message::Camera,
                        )
                        .width(Length::FillPortion(50)),
                    )
                    .push(horizontal_space(Length::FillPortion(10)))
                    .push(
                        toggler("Hide Cam".to_string(), hide_camera, Message::HideCamera)
                            .size(24)
                            .spacing(2)
                            .width(Length::FillPortion(40)),
                    )
                    .padding(1),
            )),
    )
    .padding(40)
    .center_x()
    .center_y();

    let start_button_row = Container::new(toggle_start)
        .width(Length::Fill)
        .align_x(Horizontal::Center);

    let controls_row = Container::new(
        Row::new()
            .push(Container::new(camera_row).width(Length::FillPortion(5)))
            .push(Container::new(sliders_row).width(Length::FillPortion(5))),
    );

    let body = Container::new(
        Column::new()
            .width(Length::Fill)
            .push(vertical_space(Length::Units(40)))
            .push(Container::new(
                Row::new()
                    .push(horizontal_space(Length::FillPortion(2)))
                    .push(
                        button(text("Reset to Default").size(15))
                            .on_press(Message::DefaultSettings),
                    )
                    .push(horizontal_space(Length::Units(40))),
            ))
            .push(controls_row.width(Length::FillPortion(50)))
            .push(start_button_row.width(Length::FillPortion(50))),
    )
    .height(Length::FillPortion(HEIGHT_BODY));

    let footer = footer();

    Column::new().spacing(10).push(body).push(footer)
}

fn footer() -> Container<'static, Message, Renderer> {
    // let github_button = button(
    //     text("Github")
    //         .size(5)
    //         .horizontal_alignment(alignment::Horizontal::Center)
    //         .vertical_alignment(alignment::Vertical::Center),
    // )
    // .height(Length::Units(35))
    // .width(Length::Units(35))
    // .on_press(Message::OpenGithub);

    // let logs_button = button(
    //     text("Open Logs")
    //         .size(5)
    //         .horizontal_alignment(alignment::Horizontal::Center)
    //         .vertical_alignment(alignment::Vertical::Center),
    // )
    // .height(Length::Units(35))
    // .width(Length::Units(35))
    // .on_press(Message::OpenLogs);

    let footer_row = Row::new()
        .align_items(Alignment::Center)
        .push(Text::new(format!(
            "{} v{} by {} ",
            APP_NAME, APP_VERSION, APP_AUTHORS
        )))
        // .push(github_button)
        // .push(logs_button)
        ;

    Container::new(footer_row)
        .width(Length::Fill)
        .height(Length::FillPortion(HEIGHT_FOOTER))
        .align_y(Vertical::Bottom)
        .align_x(Horizontal::Center)
        .padding(20)
}
