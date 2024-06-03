// The UI of the application

use std::{borrow::Cow, sync::atomic::Ordering};

use iced::{
    alignment::{self, Horizontal, Vertical},
    widget::{
        button, horizontal_space, pick_list, slider, text, text_input, toggler, vertical_space,
        Column, Container, Row, Text,
    },
    Alignment, Length, Renderer,
};
use opencv::{
    imgcodecs,
    types::{VectorOfi32, VectorOfu8},
};
use opencv::prelude::VectorToVec;

use crate::{consts::NO_VIDEO_IMG, enums::message::Message, structs::app::HeadTracker};

use super::style::{HEIGHT_BODY, HEIGHT_FOOTER};
use crate::consts::{APP_AUTHORS, APP_NAME, APP_REPOSITORY, APP_VERSION, ICONS};

pub fn run_page(headtracker: &HeadTracker) -> Column<Message> {
    // Convert the min_cutoff and beta values to u32
    let min_cutoff = {
        if (headtracker.config.min_cutoff.load(Ordering::SeqCst) - 0.).abs() < f32::EPSILON {
            0
        } else {
            (1. / headtracker.config.min_cutoff.load(Ordering::SeqCst)).sqrt() as u32
        }
    };

    let beta = {
        if (headtracker.config.beta.load(Ordering::SeqCst) - 0.).abs() < f32::EPSILON {
            0
        } else {
            (1. / headtracker.config.beta.load(Ordering::SeqCst)).sqrt() as u32
        }
    };
    let fps = headtracker.config.fps.load(Ordering::SeqCst);

    let ip = headtracker.config.ip.as_str();
    let port = headtracker.config.port.as_str();
    let hide_camera = headtracker.config.hide_camera;

    // Create the sliders
    let min_cutoff_slider = slider(0..=50, min_cutoff, Message::MinCutoffSliderChanged).step(1);
    let beta_slider = slider(0..=50, beta, Message::BetaSliderChanged).step(1);
    let fps_slider = slider(15..=120, fps, Message::FPSSliderChanged).step(1);

    // The main Start/Stop button
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
        .height(Length::Fixed(40.))
        .width(Length::Fixed(180.))
        .on_press(Message::Toggle)
    };

    let sliders_row = Container::new(
        Column::new()
            .push(text("Filter Settings").size(15))
            .push(vertical_space(Length::Fixed(20.)))
            .push(text("Speed").size(14))
            .push(Container::new(min_cutoff_slider).width(Length::FillPortion(2)))
            .push(vertical_space(Length::Fixed(10.)))
            .push(text("Smooth").size(14))
            .push(Container::new(beta_slider).width(Length::FillPortion(2)))
            .push(vertical_space(Length::Fixed(30.)))
            .push(text("FPS").size(15))
            .push(Container::new(fps_slider).width(Length::FillPortion(2)))
            .push(vertical_space(Length::Fixed(30.)))
            .push(text("IP and Port").size(15))
            // ! IPV4 and V6 support for external devices, having only two inputs, ip and port
            .push(Container::new(
                Row::new()
                    .spacing(5)
                    .push(
                        text_input("127.0.0.1", ip)
                            .on_input(Message::InputIP)
                            .width(Length::FillPortion(70)),
                    )
                    .push(text("      "))
                    .push(
                        text_input("4242", port)
                            .on_input(Message::InputPort)
                            .width(Length::FillPortion(15)),
                    ),
            ))
            .push(vertical_space(Length::Fixed(30.))),
    )
    .padding(40);

    // If camera is set to hidden, show a placeholder image
    let image = match hide_camera {
        true => NO_VIDEO_IMG.to_vec(),
        false => {
            if headtracker.headtracker_running.load(Ordering::SeqCst) {
                let frame = headtracker.frame.clone();
                let mut encoded_image = VectorOfu8::new();
                let params = VectorOfi32::new();
                match imgcodecs::imencode(".PNG", &frame, &mut encoded_image, &params) {
                    Ok(_) => {}
                    Err(e) => {
                        tracing::error!("Error encoding image: {}", e);
                    }
                }
                encoded_image.to_vec()
            } else {
                NO_VIDEO_IMG.to_vec()
            }
        }
    };

    // Contains camera placeholder, available cameras list and the toggle button to hide the camera
    let camera_row = Container::new(
        Column::new()
            .push({
                iced::widget::image::viewer(iced::widget::image::Handle::from_memory(image))
                    .width(Length::Fill)
                    .height(Length::Fixed(200.))
            })
            .push(vertical_space(Length::Fixed(32.)))
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
                            Some(headtracker.config.selected_camera.clone()),
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
            .push(vertical_space(Length::Fixed(40.)))
            .push(Container::new(
                Row::new()
                    .push(horizontal_space(Length::FillPortion(2)))
                    // If there is a new release, show a button to download it/update it
                    .push(match &headtracker.release_info {
                        Some(release_info) => Container::new(
                            button(
                                text(format!(" {} now available! ", release_info.tag_name))
                                    .size(15),
                            )
                            .on_press(Message::OpenURL(release_info.html_url.clone())),
                        ),
                        None => Container::new(vertical_space(Length::Fixed(40.))),
                    })
                    .push(horizontal_space(Length::Fixed(34.)))
                    .push(
                        button(text("  Reset to Default  ").size(15))
                            .on_press(Message::DefaultSettings),
                    )
                    .push(horizontal_space(Length::Fixed(40.))),
            ))
            .push(controls_row.width(Length::FillPortion(50)))
            .push(start_button_row.width(Length::FillPortion(50)))
            .push(vertical_space(Length::Fixed(20.)))
            .push(
                Container::new(
                    Row::new()
                        .push(horizontal_space(Length::FillPortion(40)))
                        // If there is an error, show it
                        .push(
                            text(headtracker.error_tracker.clone().lock().unwrap())
                                .size(15)
                                .horizontal_alignment(Horizontal::Center)
                                .width(Length::FillPortion(50)),
                        )
                        .push(horizontal_space(Length::FillPortion(40))),
                )
                .center_x(),
            ),
    )
    .height(Length::FillPortion(HEIGHT_BODY));

    let footer = footer();

    Column::new().spacing(10).push(body).push(footer)
}

// Shows app version and links to github and logs
fn footer() -> Container<'static, Message, Renderer> {
    let github_button = button(
        Text::new('\u{48}'.to_string())
            .font(ICONS)
            .size(14.)
            .horizontal_alignment(alignment::Horizontal::Center)
            .vertical_alignment(alignment::Vertical::Center),
    )
    .height(Length::Fixed(35.))
    .width(Length::Fixed(40.))
    .on_press(Message::OpenURL(String::from(APP_REPOSITORY)));

    let logs_button = button(
        Text::new('\u{66}'.to_string())
            .font(ICONS)
            .size(14.)
            .horizontal_alignment(alignment::Horizontal::Center)
            .vertical_alignment(alignment::Vertical::Center),
    )
    .height(Length::Fixed(35.))
    .width(Length::Fixed(40.))
    .on_press(Message::OpenLogs);

    let footer_row = Row::new()
        .align_items(Alignment::Center)
        .push(Text::new(format!(
            "{} v{} by {}     ",
            APP_NAME, APP_VERSION, APP_AUTHORS
        )))
        .push(github_button)
        .push(horizontal_space(Length::Fixed(10.)))
        .push(logs_button);

    Container::new(footer_row)
        .width(Length::Fill)
        .height(Length::FillPortion(HEIGHT_FOOTER))
        .align_y(Vertical::Bottom)
        .align_x(Horizontal::Center)
        .padding(20)
}
