use std::{borrow::Cow, sync::atomic::Ordering};

use iced::{
    alignment::{self, Horizontal, Vertical},
    widget::{
        button, horizontal_space, pick_list, rule, slider, text, text_input, vertical_space,
        Column, Container, Row, Text,
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

    let input_ip_0 = headtracker.ip_arr_0.as_str();
    let input_ip_1 = headtracker.ip_arr_1.as_str();
    let input_ip_2 = headtracker.ip_arr_2.as_str();
    let input_ip_3 = headtracker.ip_arr_3.as_str();
    let port = headtracker.port.as_str();

    let min_cutoff_slider = slider(0..=50, min_cutoff, Message::MinCutoffSliderChanged);
    let beta_slider = slider(0..=50, beta, Message::BetaSliderChanged);
    let fps_slider = slider(15..=120, fps, Message::FPSSliderChanged);

    let toggle_start = {
        let label = match headtracker.keep_running.load(Ordering::SeqCst) {
            true => "Stop",
            false => "Start",
        };

        button(text(label)).on_press(Message::Toggle)
    };

    let sliders_row = Container::new(
        Column::new()
            .push(Container::new(
                Row::new()
                    .push(horizontal_space(Length::FillPortion(50)))
                    .push(button(text("Reset to Default").size(15)).on_press(Message::Toggle))
                    .width(Length::FillPortion(50)),
            ))
            .push(vertical_space(Length::Units(30)))
            .push(text("Filter Settings").size(15))
            .push(Container::new(min_cutoff_slider).width(Length::FillPortion(2)))
            .push(vertical_space(Length::Units(10)))
            .push(Container::new(beta_slider).width(Length::FillPortion(2)))
            .push(vertical_space(Length::Units(30)))
            .push(text("FPS").size(15))
            .push(Container::new(fps_slider).width(Length::FillPortion(2)))
            .push(vertical_space(Length::Units(30)))
            .push(text("IP and Port").size(15))
            .push(Container::new(
                Row::new()
                    .spacing(5)
                    .push(
                        text_input("127", input_ip_0, Message::InputIP0)
                            .width(Length::FillPortion(10)),
                    )
                    .push(
                        text_input("0", input_ip_1, Message::InputIP1)
                            .width(Length::FillPortion(5)),
                    )
                    .push(
                        text_input("0", input_ip_2, Message::InputIP2)
                            .width(Length::FillPortion(5)),
                    )
                    .push(
                        text_input("1", input_ip_3, Message::InputIP3)
                            .width(Length::FillPortion(5)),
                    )
                    .push(text("      "))
                    .push(
                        text_input("4242", port, Message::InputPort).width(Length::FillPortion(15)),
                    ),
            ))
            .push(vertical_space(Length::Units(30)))
            .push(Container::new(
                Row::new()
                    .push(horizontal_space(Length::FillPortion(10)))
                    .push(
                        button(text("Save").horizontal_alignment(Horizontal::Center))
                            .width(Length::Fill)
                            .on_press(Message::Toggle)
                            .width(Length::FillPortion(80)),
                    )
                    .push(horizontal_space(Length::FillPortion(10))),
            ))
            .push(Container::new(
                Row::new()
                    .push(horizontal_space(Length::FillPortion(26)))
                    .push(
                        text("Unsaved changes")
                            .size(15)
                            .width(Length::FillPortion(48)),
                    )
                    .push(horizontal_space(Length::FillPortion(26))),
            )),
    )
    .padding(40);

    let camera_row = Container::new(
        Row::new().push(pick_list(
            Cow::from(
                headtracker
                    .camera_list
                    .keys()
                    .cloned()
                    .collect::<Vec<String>>(),
            ),
            headtracker.selected_camera.clone(),
            Message::Camera,
        )), // .push(
            //     Container::new(button(text("Hide Camera")).on_press(Message::Toggle))
            //         .width(Length::FillPortion(5)),
            // ),
    )
    .padding(40);

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
            .push(controls_row)
            .push(start_button_row),
    )
    .height(Length::FillPortion(HEIGHT_BODY));

    let footer = footer();

    Column::new().spacing(10).push(body).push(footer)
}

fn footer() -> Container<'static, Message, Renderer> {
    let github_button = button(
        text("Github")
            .size(5)
            .horizontal_alignment(alignment::Horizontal::Center)
            .vertical_alignment(alignment::Vertical::Center),
    )
    .height(Length::Units(35))
    .width(Length::Units(35))
    .on_press(Message::OpenGithub);

    let logs_button = button(
        text("Open Logs")
            .size(5)
            .horizontal_alignment(alignment::Horizontal::Center)
            .vertical_alignment(alignment::Vertical::Center),
    )
    .height(Length::Units(35))
    .width(Length::Units(35))
    .on_press(Message::OpenLogs);

    let footer_row = Row::new()
        .align_items(Alignment::Center)
        .push(Text::new(format!(
            "{} {} by {} ",
            APP_NAME, APP_VERSION, APP_AUTHORS
        )))
        .push(github_button)
        .push(logs_button)
        .push(Text::new("  "));

    Container::new(footer_row)
        .width(Length::Fill)
        .height(Length::FillPortion(HEIGHT_FOOTER))
        .align_y(Vertical::Bottom)
        .align_x(Horizontal::Center)
        .padding(20)
}
