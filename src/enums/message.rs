use iced_native::Event;

#[derive(Debug, Clone)]
pub enum Message {
    Toggle,
    MinCutoffSliderChanged(u32),
    BetaSliderChanged(u32),
    FPSSliderChanged(u32),
    InputIP0(String),
    InputIP1(String),
    InputIP2(String),
    InputIP3(String),
    InputPort(String),
    Camera(String),
    OpenGithub,
    OpenLogs,
    EventOccurred(Event),
}
