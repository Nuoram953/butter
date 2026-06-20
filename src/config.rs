use anyhow::{Context, Result};
use directories::ProjectDirs;
use serde::Deserialize;
use std::{fs, io, path::PathBuf};

use crate::rules::file::FileRuleConfig;

const APP_NAME: &str = "butter";
const RULES_FILE: &str = "rules.yml";

// ======================================================
// CONFIG ROOT (matches YAML structure)
// ======================================================

#[derive(Debug, Deserialize)]
pub struct Config {
    pub rules: Vec<RuleConfig>,
}

// ======================================================
// RULE CONFIG ENUM (YAML tagged union)
// ======================================================

#[derive(Debug, Deserialize)]
#[serde(rename_all = "lowercase")]
#[serde(tag = "type")]
pub enum RuleConfig {
    File(FileRuleConfig),
}

// ======================================================
// FILE SYSTEM / CONFIG LOCATION
// ======================================================

pub fn config_dir() -> io::Result<PathBuf> {
    let project_dirs = ProjectDirs::from("", "", APP_NAME).ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            "Unable to determine config directory",
        )
    })?;

    Ok(project_dirs.config_dir().to_path_buf())
}

pub fn rules_file_path() -> io::Result<PathBuf> {
    Ok(config_dir()?.join(RULES_FILE))
}

pub fn ensure_rules_file() -> io::Result<PathBuf> {
    let dir = config_dir()?;
    fs::create_dir_all(&dir)?;

    let path = dir.join(RULES_FILE);

    if !path.exists() {
        fs::write(&path, DEFAULT_RULES)?;
    }

    Ok(path)
}

// ======================================================
// LOAD / SAVE CONFIG
// ======================================================

pub fn load_config() -> Result<Config> {
    let path = ensure_rules_file().context("failed to ensure rules file exists")?;

    let contents = fs::read_to_string(&path).context("failed to read rules file")?;

    let config: Config = serde_yaml::from_str(&contents).context("failed to parse YAML config")?;

    Ok(config)
}
// ======================================================
// DEFAULT CONFIG FILE
// ======================================================

const DEFAULT_RULES: &str = r#"
rules:
  - name: deploy_change_requires_traffic
    type: file
    match:
      - deploy
      - scripts/deploy
    warning: "Deploy script changed → did you update traffic script?"
"#;
