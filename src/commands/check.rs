use anyhow::{Ok, Result};
use log::info;

use crate::{
    config::{self},
    output::printer::Printer,
    rules::result::RuleResult,
};

pub fn handle(branch: Option<&str>) -> Result<()> {
    let mut rule_results: Vec<RuleResult> = Vec::new();
    let config = config::load_config()?;
    let printer = Printer::new();

    for rule in config.rules {
        info!("Checking for {:#?}", rule);

        let result = rule.evaluate(branch);

        printer.print_rule_result(&result);

        rule_results.push(result);
    }

    printer.print_end_results(rule_results);

    Ok(())
}
