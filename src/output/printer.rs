use crate::rules::result::{RuleResult, Status};
use colored::Colorize;

pub struct Printer {
    width: usize,
}

impl Default for Printer {
    fn default() -> Self {
        Self { width: 80 }
    }
}

impl Printer {
    pub fn new() -> Self {
        Self { width: 80 }
    }

    pub fn print_rule_result(&self, result: &RuleResult) {
        println!("{}", self.format_result(result))
    }

    fn format_result(&self, result: &RuleResult) -> String {
        let dots = self.dot_leader(&result.name, &result.status);

        let output = format!(
            "{} {} {}",
            result.name.cyan(),
            dots,
            result.status.as_colored_str()
        );

        output
    }

    pub fn print_end_results(&self, results: Vec<RuleResult>) {
        let failures: Vec<&RuleResult> = results
            .iter()
            .filter(|result| result.status != Status::Success)
            .collect();

        if failures.is_empty() {
            return;
        }

        println!("\n{}", "=".repeat(self.width));
        println!("{}", "FAILURES".bold());
        println!("{}", "=".repeat(self.width));

        for result in &failures {
            println!(
                "\n{} {}",
                result.name.cyan(),
                result.status.as_colored_str()
            );

            for (index, failure) in result.failures.iter().enumerate() {
                let file_display = failure
                    .file
                    .as_ref()
                    .and_then(|f| f.to_str())
                    .unwrap_or("<unknown file>");

                println!("  {}. {}", index + 1, file_display);
                println!("     ↳ {}", failure.reason.yellow());
            }
        }

        println!("\n{}", "-".repeat(self.width));
        let total_failures: usize = failures.iter().map(|r| r.failures.len()).sum();
        println!(
            "{} rule(s) failed, {} failure(s) total",
            failures.len(),
            total_failures
        );
        println!("{}", "=".repeat(self.width));
    }

    fn dot_leader(&self, rule_name: &str, status: &Status) -> String {
        let dots = self
            .width
            .saturating_sub(rule_name.len() + status.to_string().len() + 2);

        ".".repeat(dots)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_success_without_message() {
        let printer = Printer::new();

        let result = RuleResult {
            name: "test".into(),
            status: Status::Success,
            failures: Vec::new(),
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
            name: "test".into(),
            status: Status::Warning,
            failures: Vec::new(),
        };

        let output = printer.format_result(&result);

        println!("{}", output);

        assert!(output.contains("test"));
        assert!(output.contains("Warning"));
    }

    #[test]
    fn formats_error_with_message() {
        let printer = Printer::new();

        let result = RuleResult {
            name: "test".into(),
            status: Status::Error,
            failures: Vec::new(),
        };

        let output = printer.format_result(&result);

        println!("{}", output);

        assert!(output.contains("test"));
        assert!(output.contains("Error"));
    }
}
