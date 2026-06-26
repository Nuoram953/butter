use regex::Regex;
use schemars::JsonSchema;
use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::{config, git, rules::Level};

/// Checks that filenames in a given directory match a naming pattern (regex).
#[derive(Debug, Deserialize, JsonSchema)]
pub struct FileNameRuleConfig {
    /// Name of the rule.
    pub name: String,

    /// Directory to search.
    pub directory: String,

    /// Regular expression used to match files.
    pub pattern: String,

    /// Message displayed when the rule fails.
    pub message: String,

    /// Severity of the rule.
    pub level: Level,
}

impl Default for FileNameRuleConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            directory: String::new(),
            pattern: String::new(),
            message: String::new(),
            level: Level::Warn,
        }
    }
}

impl FileNameRuleConfig {
    pub fn evaluate_files(&self, files: &[PathBuf]) -> bool {
        let re = Regex::new(&self.pattern).expect("invalid regex");
        files
            .iter()
            .filter(|file| file.to_str().unwrap().contains(&self.directory))
            .all(|file| {
                let stem = Path::new(file)
                    .file_stem()
                    .and_then(|s| s.to_str())
                    .unwrap_or("");
                re.is_match(stem)
            })
    }

    pub fn evaluate(&self, branch: Option<&str>) -> bool {
        let config = config::load_config();
        let files = git::get_changed_files(branch.unwrap_or(&config.unwrap().default_branch));
        self.evaluate_files(&files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;

    #[test]
    fn returns_true_when_no_when_match() {
        let rule = FileNameRuleConfig {
            directory: String::from("src/migrations"),
            pattern: String::from(r"^\d{4}_[A-Z]+_\d+_[a-z0-9_]+$"),
            ..Default::default()
        };

        let files = vec![PathBuf::from("README.md")];

        assert!(rule.evaluate_files(&files));
    }

    #[test]
    fn returns_true_when_match_and_valid() {
        let rule = FileNameRuleConfig {
            directory: String::from("src/migrations"),
            pattern: String::from(r"^\d{4}_[A-Z]+_\d+_[a-z0-9_]+$"),
            ..Default::default()
        };

        let files = vec![PathBuf::from("src/migrations/0001_PROJ_1234_add_login.sql")];

        assert!(rule.evaluate_files(&files));
    }

    #[test]
    fn returns_false_when_match_and_invalid() {
        let rule = FileNameRuleConfig {
            directory: String::from("src/migrations"),
            pattern: String::from(r"^\d{4}_[A-Z]+_\d+_[a-z0-9_]+$"),
            ..Default::default()
        };

        let files = vec![PathBuf::from("src/migrations/PROJ_1234_add_login.sql")];

        assert!(!rule.evaluate_files(&files));
    }

    #[test]
    fn returns_false_when_at_least_one_fails() {
        let rule = FileNameRuleConfig {
            directory: String::from("src/migrations"),
            pattern: String::from(r"^\d{4}_[A-Z]+_\d+_[a-z0-9_]+$"),
            ..Default::default()
        };

        let files = vec![
            PathBuf::from("src/migrations/PROJ_1234_add_login.sql"),
            PathBuf::from("src/migrations/0001_PROJ_1234_add_login.sql"),
        ];

        assert!(!rule.evaluate_files(&files));
    }
}
