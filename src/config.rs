use directories::ProjectDirs;
use serde::{Deserialize, Serialize};
use std::{
    fs, io,
    path::{Path, PathBuf},
};

use anyhow::Result;

const APP_NAME: &str = "missed-checks";
const RULES_FILE: &str = "rules.yml";

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    pub rules: Vec<Rule>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Rule {
    pub name: String,

    #[serde(rename = "type")]
    pub rule_type: RuleType,

    #[serde(rename = "match")]
    pub patterns: Vec<String>,

    pub warning: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum RuleType {
    File,
}

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
    let config_dir = config_dir()?;

    fs::create_dir_all(&config_dir)?;

    let rules_file = config_dir.join(RULES_FILE);

    if !rules_file.exists() {
        fs::write(&rules_file, DEFAULT_RULES)?;
    }

    Ok(rules_file)
}

pub fn load_config() -> Result<Config> {
    let path = ensure_rules_file()?;
    let contents = fs::read_to_string(path)?;

    let config = serde_yaml::from_str::<Config>(&contents)?;

    Ok(config)
}

pub fn save_config(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    let path = ensure_rules_file()?;

    let yaml = serde_yaml::to_string(config)?;
    fs::write(path, yaml)?;

    Ok(())
}

const DEFAULT_RULES: &str = r#"
rules:
  - name: deploy_change_requires_traffic
    type: file
    match:
      - deploy
      - scripts/deploy
    warning: "Deploy script changed → did you update traffic script?"
"#;
