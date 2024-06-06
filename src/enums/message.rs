// Events than can be triggered by the user in the GUI

use iced::event::{Event};

#[derive(Debug, Clone)]
pub enum Message {
    Toggle,
    DefaultSettings,
    Tick,
    MinCutoffSliderChanged(u32),
    BetaSliderChanged(u32),
    FPSSliderChanged(u32),
    InputIP(String),
    InputPort(String),
    Camera(String),
    HideCamera(bool),
    OpenURL(String),
    OpenLogs,
    EventOccurred(Event),
}
