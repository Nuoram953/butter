use anyhow::{Ok, Result};
use colored::Colorize;
use log::debug;

use crate::{
    config::{self, RuleConfig},
    rules::{Level, Rule},
};

pub fn handle() -> Result<()> {
    let config = config::load_config()?;

    for rule in config.rules {
        debug!("Checking for {:#?}", rule);
        let passed = match rule {
            RuleConfig::File(ref r) => r.evaluate(),
        };

        print_result(&rule, passed);
    }

    Ok(())
}

fn print_result(rule: &RuleConfig, passed: bool) {
    const WIDTH: usize = 80;

    let (name, message, level) = match rule {
        RuleConfig::File(r) => (&r.name, &r.message, &r.level),
    };

    let result_text = match passed {
        true => "success".green(),
        false => match level {
            Level::Warn => "warning".yellow(),
            Level::Error => "failure".red(),
        },
    };

    let dots = WIDTH.saturating_sub(
        name.len() + result_text.len() + 2, // spaces around dots
    );

    let dots = ".".repeat(dots);

    println!("{} {} {}", name, dots, result_text);

    if !passed {
        println!("  {}", message.yellow());
    }
}
