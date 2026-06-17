use serde::{Deserialize, Serialize};

pub mod file;

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Level {
    Warn,
    Error,
}

pub trait Rule {
    fn name(&self) -> &str;
    fn evaluate(&self) -> bool;
}
