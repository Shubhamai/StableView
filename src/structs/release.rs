// Contains new release information

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub struct Release {
    pub name: String,
    pub tag_name: String,
    pub html_url: String,
}
