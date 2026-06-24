use std::path::PathBuf;

use serde::Deserialize;

use crate::{config, git, rules::Level};

#[derive(Debug, Deserialize)]
pub struct FileRuleConfig {
    pub name: String,
    pub when: Vec<String>,
    #[serde(default)]
    pub unless: Vec<String>,
    pub message: String,
    pub level: Level,
}

impl Default for FileRuleConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            when: Vec::new(),
            unless: Vec::new(),
            message: String::new(),
            level: Level::Warn,
        }
    }
}

impl FileRuleConfig {
    pub fn evaluate_files(&self, files: &[PathBuf]) -> bool {
        let file_match_when = files.iter().any(|file| {
            self.when
                .iter()
                .any(|pattern| file.to_str().unwrap_or("").contains(pattern))
        });

        if !file_match_when {
            return true;
        }

        let file_match_unless = files.iter().any(|file| {
            self.unless
                .iter()
                .any(|pattern| file.to_str().unwrap_or("").contains(pattern))
        });

        file_match_unless
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
        let rule = FileRuleConfig {
            when: vec!["src".into()],
            unless: vec!["test".into()],
            ..Default::default()
        };

        let files = vec![PathBuf::from("README.md")];

        assert!(rule.evaluate_files(&files));
    }

    #[test]
    fn returns_false_when_when_matches_but_unless_does_not() {
        let rule = FileRuleConfig {
            when: vec!["src".into()],
            unless: vec!["test".into()],
            ..Default::default()
        };

        let files = vec![PathBuf::from("src/main.rs")];

        assert!(!rule.evaluate_files(&files));
    }

    #[test]
    fn returns_true_when_both_when_and_unless_match() {
        let rule = FileRuleConfig {
            when: vec!["src".into()],
            unless: vec!["test".into()],
            ..Default::default()
        };

        let files = vec![PathBuf::from("src/main.rs"), PathBuf::from("test/data.txt")];

        assert!(rule.evaluate_files(&files));
    }
}
