use anyhow::{Ok, Result};
use log::debug;

use crate::{
    config::{self},
    output::printer::Printer,
};

pub fn handle() -> Result<()> {
    let config = config::load_config()?;

    for rule in config.rules {
        debug!("Checking for {:#?}", rule);

        let passed = rule.evaluate();

        let printer = Printer::new();

        printer.print_result(&rule, passed);
    }

    Ok(())
}
