use anyhow::{Ok, Result};

use crate::config;

pub fn handle() -> Result<()> {
    let config = config::load_config()?;

    for rule in config.rules {
        println!("{:#?}", rule)
    }

    Ok(())
}
