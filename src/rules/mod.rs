use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub mod file;
pub mod file_name;
pub mod result;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Warn,
    Error,
}
