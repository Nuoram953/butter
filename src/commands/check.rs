use anyhow::{Ok, Result};
use log::debug;

use crate::{
    config::{self},
    output::printer::Printer,
    rules::result::to_result,
};

pub fn handle(branch: Option<&str>) -> Result<()> {
    let config = config::load_config()?;

    for rule in config.rules {
        debug!("Checking for {:#?}", rule);

        let passed = rule.evaluate(branch);

        let result = to_result(&rule, passed);

        let printer = Printer::new();

        printer.print_result(result);
    }

    Ok(())
}
