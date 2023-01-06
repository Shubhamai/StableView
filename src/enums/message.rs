use iced_native::Event;

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
    OpenGithub,
    OpenLogs,
    EventOccurred(Event),
}
