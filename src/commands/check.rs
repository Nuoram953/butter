use std::process;

use anyhow::{Ok, Result};
use log::{debug, info};

use crate::{
    config::{self},
    output::printer::Printer,
    rules::result::RuleResult,
};

pub fn handle(branch: Option<&str>) -> Result<()> {
    let mut rule_results: Vec<RuleResult> = Vec::new();
    let config = config::load_config()?;
    let printer = Printer::new();

    info!(
        "Evaluating {} rule(s) against branch {:?}",
        config.rules.len(),
        branch
    );

    for rule in config.rules {
        debug!("Checking rule {:#?}", rule);

        let result = rule.evaluate(branch);

        debug!("Rule result {:?}", result);

        printer.print_rule_result(&result);

        rule_results.push(result);
    }

    let failed = rule_results
        .iter()
        .filter(|r| r.status != crate::rules::result::Status::Success)
        .count();
    info!(
        "Done: {} rule(s) flagged out of {}",
        failed,
        rule_results.len()
    );

    printer.print_end_results(rule_results);

    if failed > 0 {
        process::exit(1)
    }

    Ok(())
}
