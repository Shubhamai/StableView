#[derive(Debug, Clone)]
pub enum Message {
    Toggle,
    MinCutoffSliderChanged(u32),
    BetaSliderChanged(u32),
    InputIP(String),
    OpenGithub,
    OpenLogs,
}
