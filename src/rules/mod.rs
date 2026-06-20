use serde::{Deserialize, Serialize};

pub mod file;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Warn,
    Error,
}
