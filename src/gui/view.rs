use std::{borrow::Cow, io::Read, sync::atomic::Ordering};

use iced::{
    alignment::{self, Horizontal, Vertical},
    theme::{self, Slider},
    widget::{
        button, column, container, horizontal_space, image, pick_list, row, slider, svg, text,
        text_input, vertical_space, Button, Column, Container, PickList, Row, Text,
    },
    window::{self, Position},
    Alignment, Application, Command, Element, Length, Renderer, Settings, Subscription, Theme,
};

use crate::{enums::message::Message, structs::app::HeadTracker};

use super::style::{APP_AUTHORS, APP_NAME, APP_VERSION, HEIGHT_BODY, HEIGHT_FOOTER, };

pub fn run_page(headtracker: &HeadTracker) -> Column<Message> {
    let input_min_cutoff = (headtracker.min_cutoff.load(Ordering::SeqCst) * 10000.) as u32;
    let input_beta = (headtracker.beta.load(Ordering::SeqCst) * 1000.) as u32;
    let input_ip = "127";

    let min_cutoff_slider = slider(0..=10000, input_min_cutoff, Message::MinCutoffSliderChanged);
    let beta_slider = slider(0..=1000, input_beta, Message::BetaSliderChanged);

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
            .push(text("IP and Port").size(15))
            .push(Container::new(
                Row::new()
                    .spacing(5)
                    .push(
                        text_input("IP", input_ip, Message::InputIP).width(Length::FillPortion(10)),
                    )
                    .push(
                        text_input("IP", input_ip, Message::InputIP).width(Length::FillPortion(5)),
                    )
                    .push(
                        text_input("IP", input_ip, Message::InputIP).width(Length::FillPortion(5)),
                    )
                    .push(
                        text_input("IP", input_ip, Message::InputIP).width(Length::FillPortion(5)),
                    )
                    .push(text("      "))
                    .push(
                        text_input("IP", input_ip, Message::InputIP).width(Length::FillPortion(15)),
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
                ),
            )
            .push(
                Container::new(button(text("Hide Camera")).on_press(Message::Toggle))
                    .width(Length::FillPortion(5)),
            ),
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
            .size(24)
            .horizontal_alignment(alignment::Horizontal::Center)
            .vertical_alignment(alignment::Vertical::Center),
    )
    .height(Length::Units(35))
    .width(Length::Units(35))
    .on_press(Message::OpenGithub);

    let logs_button = button(
        text("Open Logs")
            .size(12)
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
