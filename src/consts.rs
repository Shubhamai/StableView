pub const APP_NAME: &str = env!("CARGO_PKG_NAME");
pub const APP_VERSION: &str = env!("CARGO_PKG_VERSION");
pub const APP_REPOSITORY: &str = env!("CARGO_PKG_REPOSITORY");
pub const APP_AUTHORS: &str = env!("CARGO_PKG_AUTHORS");
pub const APP_GITHUB_API: &str =
    "https://api.github.com/repos/shubhamai/stableview/releases/latest";

pub const MODEL: &[u8] = include_bytes!("../assets/model/mb05_120x120.onnx");
pub const DATA: &[u8] = include_bytes!("../assets/model/data.json");
pub const BLAZE_FACE_MODEL: &[u8] = include_bytes!("../assets/model/blazeface-320.onnx");

pub const ICON: &[u8] = include_bytes!("../assets/brand/Product.ico");
pub const INTER_FONT: &[u8] = include_bytes!("../assets/fonts/Inter-Regular.ttf");
pub const NO_VIDEO_IMG: &[u8] = include_bytes!("../assets/brand/no_video.png");

pub const ICONS_FONT: &[u8] = include_bytes!("../assets/fonts/icons.ttf");
