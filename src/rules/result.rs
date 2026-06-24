use crate::{config::RuleConfig, rules::Level};
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

pub struct RuleResult {
    pub rule_name: String,
    pub message: Option<String>,
    pub status: Status,
}

pub fn to_result(rule: &RuleConfig, passed: bool) -> RuleResult {
    let status = match (passed, rule.level()) {
        (true, _) => Status::Success,
        (false, Level::Warn) => Status::Warning,
        (false, Level::Error) => Status::Error,
    };

    let message = if matches!(status, Status::Success) {
        None
    } else {
        Some(rule.message().to_string())
    };

    RuleResult {
        rule_name: rule.name().to_string(),
        status,
        message,
    }
}
