use crate::rules::result::{RuleResult, Status};
use colored::Colorize;

pub struct Printer {
    width: usize,
}

impl Printer {
    pub fn new() -> Self {
        Self { width: 80 }
    }

    pub fn print_result(&self, result: RuleResult) {
        println!("{}", self.format_result(&result))
    }

    fn format_result(&self, result: &RuleResult) -> String {
        let dots = self.dot_leader(&result.rule_name, &result.status);

        let mut output = format!(
            "{} {} {}",
            result.rule_name.cyan(),
            dots,
            result.status.as_colored_str()
        );

        if matches!(result.status, Status::Warning | Status::Error) {
            output.push_str(&format!(
                "\n    ↳ {}",
                result.message.as_ref().unwrap().yellow()
            ));
        }

        output
    }

    fn dot_leader(&self, rule_name: &str, status: &Status) -> String {
        let dots = self
            .width
            .saturating_sub(rule_name.len() + status.to_string().len() + 2);

        return ".".repeat(dots);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_success_without_message() {
        let printer = Printer::new();

        let result = RuleResult {
            rule_name: "test".into(),
            status: Status::Success,
            message: None,
        };

        let output = printer.format_result(&result);

        println!("{}", output);

        assert!(output.contains("test"));
        assert!(output.contains("Success"));
    }

    #[test]
    fn formats_warning_with_message() {
        let printer = Printer::new();

        let result = RuleResult {
            rule_name: "test".into(),
            status: Status::Warning,
            message: Some("message".to_string()),
        };

        let output = printer.format_result(&result);

        println!("{}", output);

        assert!(output.contains("test"));
        assert!(output.contains("Warning"));
        assert!(output.contains("message"));
    }

    #[test]
    fn formats_error_with_message() {
        let printer = Printer::new();

        let result = RuleResult {
            rule_name: "test".into(),
            status: Status::Error,
            message: Some("message".to_string()),
        };

        let output = printer.format_result(&result);

        println!("{}", output);

        assert!(output.contains("test"));
        assert!(output.contains("Error"));
        assert!(output.contains("message"));
    }
}
