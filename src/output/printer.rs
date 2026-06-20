use crate::{config::RuleConfig, rules::Level};
use colored::Colorize;

pub struct Printer {
    width: usize,
}

impl Printer {
    pub fn new() -> Self {
        Self { width: 80 }
    }

    pub fn print_result(&self, rule: &RuleConfig, passed: bool) {
        match passed {
            true => self.print_line(rule, "success"),
            false => match rule.level() {
                Level::Warn => self.print_line(rule, "warning"),
                Level::Error => self.print_line(rule, "error"),
            },
        };
    }

    fn print_line(&self, rule: &RuleConfig, result: &str) {
        let rule_name = rule.name();
        let dots = self.dot_leader(rule_name, result);
        let result_with_color = match result {
            "success" => "success".green(),
            "warning" => "warning".yellow(),
            "error" => "warning".red(),
            _ => "success".green(),
        };

        println!("{} {} {}", rule_name.cyan(), dots, result_with_color);

        if result == "warning" || result == "error" {
            self.print_failure_message(rule);
        }
    }

    fn print_failure_message(&self, rule: &RuleConfig) {
        match rule.level() {
            Level::Warn => println!("    ↳ {}", rule.message().yellow()),
            Level::Error => println!("    ↳ {}", rule.message().red()),
        }
    }
    fn dot_leader(&self, rule_name: &str, result: &str) -> String {
        let dots = self
            .width
            .saturating_sub(rule_name.len() + result.len() + 2);

        return ".".repeat(dots);
    }
}
