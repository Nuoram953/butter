use std::path::PathBuf;

use schemars::JsonSchema;
use serde::Deserialize;

use crate::{
    config, git,
    rules::{
        Level,
        result::{Failure, RuleResult, get_rule_result_status},
    },
};

/// Fails if any changed file matches a `when` pattern unless a changed file also matches a corresponding `unless` pattern (e.g. "editing `src` requires also editing `test`").
#[derive(Debug, Deserialize, JsonSchema)]
pub struct FileRuleConfig {
    /// Name of the rule.
    pub name: String,

    /// Pattern if any changed file path contains one of these, the rule is triggered.
    pub when: Vec<String>,

    /// If the rule is triggered, at least one changed file must match one of these for the rule to pass. Defaults to empty if omitted.
    #[serde(default)]
    pub unless: Vec<String>,

    /// Message displayed when the rule fails.
    pub message: String,

    /// Severity of the rule.
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
    pub fn evaluate_files(&self, files: &[PathBuf]) -> RuleResult {
        let mut failures: Vec<Failure> = Vec::new();

        let any_file_matches_unless = files.iter().any(|file| {
            self.unless
                .iter()
                .any(|pattern| file.to_str().unwrap_or("").contains(pattern))
        });

        files.iter().for_each(|file| {
            let match_when_pattern = self
                .when
                .iter()
                .any(|pattern| file.to_str().unwrap_or("").contains(pattern));

            if match_when_pattern && !any_file_matches_unless {
                failures.push(Failure {
                    file: Some(file.clone()),
                    reason: String::from(&self.message),
                });
            }
        });
        RuleResult {
            name: self.name.clone(),
            status: get_rule_result_status(failures.len(), &self.level),
            failures,
        }
    }

    pub fn evaluate(&self, branch: Option<&str>) -> RuleResult {
        let config = config::load_config();
        let files = git::get_changed_files(branch.unwrap_or(&config.unwrap().default_branch));
        self.evaluate_files(&files)
    }
}

#[cfg(test)]
mod tests {
    use crate::rules::result::Status;

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

        let result = rule.evaluate_files(&files);

        assert_eq!(result.status, Status::Success);
        assert_eq!(result.failures.len(), 0);
    }

    #[test]
    fn returns_false_when_when_matches_but_unless_does_not() {
        let rule = FileRuleConfig {
            when: vec!["src".into()],
            unless: vec!["test".into()],
            ..Default::default()
        };

        let files = vec![PathBuf::from("src/main.rs")];

        let result = rule.evaluate_files(&files);

        assert_eq!(result.status, Status::Warning);
        assert_eq!(result.failures.len(), 1);
    }

    #[test]
    fn returns_true_when_both_when_and_unless_match() {
        let rule = FileRuleConfig {
            when: vec!["main.rs".into()],
            unless: vec!["data.txt".into()],
            ..Default::default()
        };

        let files = vec![PathBuf::from("src/main.rs"), PathBuf::from("test/data.txt")];

        let result = rule.evaluate_files(&files);

        assert_eq!(result.status, Status::Success);
        assert_eq!(result.failures.len(), 0);
    }
}
