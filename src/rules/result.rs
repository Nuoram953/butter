use std::path::PathBuf;

use crate::rules::Level;
use colored::{ColoredString, Colorize};
use strum_macros::{Display, EnumString};

#[derive(Debug, PartialEq, EnumString, Display)]
pub enum Status {
    Success,
    Warning,
    Error,
}

impl Status {
    pub fn as_colored_str(&self) -> ColoredString {
        match self {
            Status::Success => Status::Success.to_string().green(),
            Status::Warning => Status::Warning.to_string().yellow(),
            Status::Error => Status::Error.to_string().red(),
        }
    }
}

#[derive(Debug)]
pub struct Failure {
    pub file: Option<PathBuf>,
    pub reason: String,
}

#[derive(Debug)]
pub struct RuleResult {
    pub name: String,
    pub status: Status,
    pub failures: Vec<Failure>,
}

pub fn get_rule_result_status(failures: usize, level: &Level) -> Status {
    if failures == 0 {
        Status::Success
    } else {
        match level {
            Level::Warn => Status::Warning,
            Level::Error => Status::Error,
        }
    }
}

pub fn render_message(template: &str, vars: &[(&str, &str)]) -> String {
    let mut result = template.to_string();
    for (key, value) in vars {
        result = result.replace(&format!("{{{key}}}"), value);
    }
    result
}
